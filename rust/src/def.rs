use crate::constants::*;
use crate::yaml_components::*;
use ahash::AHashMap;
use crunch::{pack, Item, Rect, Rotation};
use gdnative::api::*;
use gdnative::prelude::*;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    hash::{Hash, Hasher},
};

#[derive(Serialize, Deserialize)]
pub struct Def {
    /// If an error occurred while making this Def. It won't be cached.
    pub corrupted: bool,

    #[serde(with = "indexmap::serde_seq")]
    pub entities_bundles: IndexMap<String, Vec<EcsComponents>>,
    pub factions: Vec<String>,
    /// The order in which mods are loaded.
    pub mod_order: Vec<ModInfo>,

    /// How many sprite atlas should exist. Used when loading from cache.
    pub num_sprite_atlases: usize,

    #[serde(skip)]
    pub vertex_def_image: Option<Ref<Image, Shared>>,
    #[serde(skip)]
    pub vertex_def_texture: Option<Ref<ImageTexture, Unique>>,
    #[serde(skip)]
    pub fragment_def_image: Option<Ref<Image, Shared>>,
    #[serde(skip)]
    pub fragment_def_texture: Option<Ref<ImageTexture, Unique>>,

    #[serde(skip)]
    pub sprite_atlases: Option<Vec<Ref<Image, Shared>>>,
    #[serde(skip)]
    pub sprite_array_texture: Option<Ref<TextureArray, Unique>>,
}
impl Def {
    /// Load a cached Def or create a new one. Return true if it is corrupted.
    pub fn load(world_path: &str, try_load_from_cache: bool, save_if_new: bool) -> Self {
        let file = gdnative::api::File::new();

        // Get mod_order.
        let mod_order = load_mod_order(world_path);

        // Hash mod_order to string. This will be the name of the cached Def.
        let mut hasher = ahash::AHasher::new_with_keys(1667, 420);
        mod_order.hash(&mut hasher);
        let hashed_mod_order = hasher.finish();

        let cache_path = format!("{}{}/", CACHE, format!("{:x}", hashed_mod_order));
        let def_path = format!("{}def", cache_path);

        // Load cached Def if it already exist.
        if try_load_from_cache {
            if file.open(&def_path, gdnative::api::File::READ).is_ok() {
                let num_byte = file.get_64();
                let data = file.get_buffer(num_byte);
                let data_read = data.read();
                if let Ok(def) = bincode::deserialize::<Def>(&data_read) {
                    // Small check to make semi sure that we did not read gibberish.
                    if def.corrupted {
                        godot_error!("Found matching cached Def, but it is corrupted. This should never happen.");
                    } else if def.mod_order != mod_order {
                        godot_error!(
                            "Found matching cached Def, but mod order don't match. Making a new one.\n{:?}\n{:?}",
                            &def.mod_order,
                            &mod_order
                        );
                    } else {
                        // Load atlases
                        if let Ok(def) = load_cached_atlas(def) {
                            file.close();
                            godot_print!("Loaded cached Def and sprite atlases.");
                            return def;
                        } else {
                            godot_warn!("Found Def, but not sprite atlas. Making a new one.");
                        }
                    }
                } else {
                    godot_error!("Found Def, but could not deserialize it. Making a new one.");
                }
            }
            // We did not find a cached Def.
            file.close();
            godot_print!("Did not find cached Def. Making and caching a new one.");
        }

        // Create a new Def.
        let def = Def::new(mod_order);

        // Save Def for faster load time.
        if save_if_new {
            if def.corrupted {
                godot_warn!("Not caching Def has it is corrupted.");
            } else {
                // Create dir.
                let dir = Directory::new();
                if let Err(err) = dir.make_dir_recursive(&cache_path) {
                    godot_error!("Could not create dir {}. {}.", &cache_path, err);
                }

                // Cache Def.
                match file.open(&def_path, File::WRITE) {
                    Ok(_) => {
                        if let Ok(data) = bincode::serialize(&def) {
                            let num_byte = i64::try_from(data.len()).unwrap_or_default();
                            file.store_64(num_byte);
                            file.store_buffer(TypedArray::from_vec(data));
                            godot_print!("Cached Def.");
                        } else {
                            godot_error!("Could not serialize Def to cache it.");
                        }
                    }
                    Err(err) => {
                        godot_error!("Could not open {} to cache Def. {}.", &def_path, err.to_string());
                    }
                } 
                file.close();

                // Cache atlases and def images.
                unsafe {
                    if let Some(img) = &def.vertex_def_image {
                        if img.assume_safe().save_png(format!("{}vertex_def.png", &cache_path)).is_err() {
                            godot_error!("Could not cache vertex_def image.");
                        }
                    }
                    if let Some(img) = &def.fragment_def_image {
                        if img.assume_safe().save_png(format!("{}fragment_def.png", &cache_path)).is_err() {
                            godot_error!("Could not cache fragment_def image.");
                        }
                    }
                    if let Some(atlases) = &def.sprite_atlases {
                        atlases.iter().enumerate().for_each(|(i, img)| {
                            if img.assume_safe().save_png(format!("{}atlas_{:02}.png", &cache_path, i)).is_err() {
                                godot_error!("Could not cache atlas image.");
                            }
                        });
                    }
                }
            }
        } else {
            godot_print!("Not caching Def.");
        }

        def
    }

    /// This does a best effort to compile all mods. Return a potentialy corrupted Def if any error where encountered.
    fn new(mod_order: Vec<ModInfo>) -> Self {
        let dir = gdnative::api::Directory::new();
        let file = gdnative::api::File::new();

        let mut corrupted = false;

        // Path are relative to mod folder.
        // Path: user://mods/my mod/ver_1_002_123/asset/mon.yaml would result in ("asset/mon.yaml", mod_id).
        let mut folder_path_to_parse = Vec::with_capacity(256);

        // All the yaml file that were found ordered by first to last mod to load.
        let mut yaml_paths = Vec::with_capacity(1024);

        // Start by adding all mod folders.
        for mod_id in 0..mod_order.len() {
            folder_path_to_parse.push(("".to_string(), mod_id));
        }

        // Find all yaml files.
        while let Some((path, mod_id)) = folder_path_to_parse.pop() {
            if dir.open(format!("{}{}", mod_order[mod_id].get_path(), &path)).is_ok() {
                if dir.list_dir_begin(true, true).is_ok() {
                    let mut file_name = dir.get_next().to_string();
                    while !file_name.is_empty() {
                        if dir.current_is_dir() {
                            godot_print!("found dir: {}", &file_name);
                            folder_path_to_parse.push((format!("{}{}/", &path, &file_name), mod_id));
                        } else if file_name.ends_with(".yaml") {
                            godot_print!("found yaml: {}", &file_name);
                            yaml_paths.push((format!("{}{}", &path, &file_name), mod_id));
                        } else if file_name.ends_with(".json") {
                            godot_print!("found json: {}", &file_name);
                            yaml_paths.push((format!("{}{}", &path, &file_name), mod_id));
                        }
                        file_name = dir.get_next().to_string();
                    }
                    dir.list_dir_end();
                } else {
                    corrupted = true;
                    godot_error!("Could not list dir for {}. Ignoring this folder.", path);
                }
            } else {
                corrupted = true;
                godot_error!("Could not open {}. Ignoring this folder.", path);
            }
        }

        // Parse all yaml into a vector of vector of YamlComponents.
        let mut list_yaml_components = Vec::with_capacity(yaml_paths.len());
        for (yaml_relative_path, mod_id) in yaml_paths.into_iter() {
            let abs_path = format!("{}{}", mod_order[mod_id].get_path(), &yaml_relative_path);
            if file.open(&abs_path, gdnative::api::File::READ).is_ok() {
                if let Ok(yaml_components) = serde_yaml::from_str::<Vec<Vec<YamlComponents>>>(&file.get_as_text().to_string()) {
                    list_yaml_components.push((yaml_components, mod_id));
                } else {
                    // Ignore this file as it can not be deserialized.
                    corrupted = true;
                    godot_error!("Could not deserialize Yaml {}. Ignoring file.", abs_path);
                }
            } else {
                corrupted = true;
                godot_error!("Could not open Yaml {}. Ignoring file.", abs_path);
            }
            file.close();
        }

        // Parse YamlComponents to EcsComponents.
        let yaml_parse_result = YamlParseResult::parse_yaml_components(list_yaml_components, &mod_order);
        godot_print!("\n{:?}\n", yaml_parse_result);

        // Sprite components reference this vector to know what id to send to the vertex shader. Negative id are flipped.
        let mut sprites_ref: Vec<[i32; 8]> = Vec::with_capacity(yaml_parse_result.sprite_paths.len());
        // sprites_ref reference this vector these images.
        let mut sprites: Vec<Ref<Image, Shared>> = Vec::with_capacity(yaml_parse_result.sprite_paths.len() * 4);

        // Add error sprite and ref.
        let err_img = Image::new();
        if err_img.load(ERR_SPRITE_PATH).is_err() {
            godot_error!("Can not load error sprite.");
        }
        sprites.push(err_img.into_shared());
        sprites_ref.push([0; 8]);

        // Gather sprite images and their rotations.
        yaml_parse_result.sprite_paths.iter().skip(1).for_each(|sprite_path| {
            // Default is fill with "error".
            let mut sp_ref = [sprites.len() as i32; 8];

            let mut rot_iter = IntoIterator::into_iter(ROT_NAME).enumerate();

            // Check for rotation sprites without _b prefix.
            let possible_rot_path = format!("{}.png", sprite_path);
            if dir.file_exists(&possible_rot_path) {
                if let Some(img) = load_image(&possible_rot_path) {
                    sp_ref[0] = sprites.len() as i32;
                    sprites.push(img);
                    // Don't search for ..._b.png
                    rot_iter.next();
                } else {
                    godot_error!("Image {} exist, but could not laod it.", possible_rot_path);
                    corrupted = true;
                }
            }

            // Check for rotation sprites with prefix.
            rot_iter.for_each(|(i, rot)| {
                let possible_rot_path = format!("{}{}.png", sprite_path, rot);
                if dir.file_exists(&possible_rot_path) {
                    if dir.file_exists(&possible_rot_path) {
                        if let Some(img) = load_image(&possible_rot_path) {
                            sp_ref[i] = sprites.len() as i32;
                            sprites.push(img);
                        }
                    } else {
                        godot_error!("Image {} exist, but could not laod it.", possible_rot_path);
                        corrupted = true;
                    }
                }
            });

            // Deal with missing rotations.
            for i in 1usize..4 {
                // Flip if missing for 1,2,3.
                if sp_ref[i] == 0 {
                    sp_ref[i] = -sp_ref[i + 4];
                }
            }
            for i in 5usize..8 {
                // Flip if missing for 5,6,7.
                if sp_ref[i] == 0 {
                    sp_ref[i] = -sp_ref[i - 4];
                }
            }
            for i in 0..8 {
                // Use closest if still missing rotation.
                if sp_ref[i] == 0 {
                    for i_replacement in ROT_REPLACEMENT[i] {
                        sp_ref[i] = sp_ref[i_replacement];
                        if sp_ref[i] != 0 {
                            break;
                        }
                    }
                }
            }

            // Check that we don't have any "error" sprite.
            for id in sp_ref {
                if id == 0 {
                    godot_error!("No sprite found for {}. This sprite will display error instead.", sprite_path);
                    corrupted = true;
                    break;
                }
            }

            sprites_ref.push(sp_ref);
        });

        // These atlases are only of size MIN_SPRITE_ATLAS_SIZE. They could be combined if we have more than MAX_SPRITE_ATLAS of them.
        // Vec<(Ref<Image, Unique>, std::vec::IntoIter<(Rect, usize)>)>
        let mut mini_sprite_atlases = Vec::with_capacity(MAX_SPRITE_ATLAS);

        let mut items_map: AHashMap<usize, usize> = AHashMap::with_capacity(sprites.len());
        let mut items: Vec<Item<usize>> = sprites
            .iter()
            .enumerate()
            .map(|(i, sprite)| {
                items_map.insert(i, i);

                Item::new(
                    i,
                    unsafe { sprite.assume_safe().get_width() as usize },
                    unsafe { sprite.assume_safe().get_height() as usize },
                    Rotation::None,
                )
            })
            .collect();

        // Pack sprites into atlases.
        let mut done_packing = false;
        while !done_packing {
            let packed = match pack(
                Rect::of_size(
                    usize::try_from(MIN_SPRITE_ATLAS_SIZE).unwrap(),
                    usize::try_from(MIN_SPRITE_ATLAS_SIZE).unwrap(),
                ),
                items.clone(),
            ) {
                Ok(all_packed) => {
                    done_packing = true;
                    all_packed.into_iter()
                }
                Err(some_packed) => some_packed.into_iter(),
            };

            let atlas_image = Image::new();
            atlas_image.create(MIN_SPRITE_ATLAS_SIZE, MIN_SPRITE_ATLAS_SIZE, false, Image::FORMAT_RGBA8);

            packed.clone().for_each(|(rect, id)| {
                // Remove from items.
                items.swap_remove(*items_map.get(&id).expect("item should be in items_map."));
                if let Some(just_swapped) = items.last() {
                    *items_map.get_mut(&just_swapped.data).expect("item should be in items_map.") = items.len() - 1;
                }
                
                // Blitz the sprites into the atlas.
                atlas_image.blit_rect(
                    &sprites[id],
                    unsafe {
                        Rect2 {
                            position: Vector2::ZERO,
                            size: sprites[id].assume_safe().get_size(),
                        }
                    },
                    Vector2::new(rect.x as f32, rect.y as f32),
                );
            });
            mini_sprite_atlases.push((atlas_image, packed));
        }

        // Make def texture. Used by vertex shader to draw the right sprite from a single u32.
        let vertex_def_image = Image::new();
        vertex_def_image.create(256, (sprites.len() as i64) / 256 + 1, false, Image::FORMAT_RGBAF);
        vertex_def_image.lock();
        // Used to tell the fragment shader which layer to sample on the sprite atlas array. 2 idx are packed in each pixel.
        let fragment_def_image = Image::new();
        fragment_def_image.create(128, (sprites.len() as i64) / 128 + 1, false, Image::FORMAT_L8);
        fragment_def_image.lock();

        // Combine mini_sprite_atlases into bigger atlas if we have more than MAX_SPRITE_ATLAS of them.
        let reduction = (mini_sprite_atlases.len() / MAX_SPRITE_ATLAS + 1).next_power_of_two();
        let sprite_atlases_size = MIN_SPRITE_ATLAS_SIZE * reduction as i64;
        let sprite_atlases: Vec<Ref<Image, Shared>>;
        if reduction > 1 {
            // TODO: We have to combine mini_sprite_atlases.
            godot_error!("Combining sprite atlas is not implemented yet.");
            todo!();
        } else {
            sprite_atlases = mini_sprite_atlases
                .into_iter()
                .enumerate()
                .map(|(i, (img, packed))| {
                    // Set def image pixel data.
                    let layer = i as f32;
                    packed.into_iter().for_each(|(rect, id)| {
                        // Def.
                        let mut id_int = id as i64;
                        let mut x = id_int % 256;
                        let mut y = id_int / 256;
                        let mut col = Color {
                            r: rect.x as f32 / sprite_atlases_size as f32,
                            g: rect.y as f32 / sprite_atlases_size as f32,
                            b: rect.w as f32 / sprite_atlases_size as f32,
                            a: rect.h as f32 / sprite_atlases_size as f32,
                        };
                        vertex_def_image.set_pixel(x, y, col);

                        // Def layer.
                        let even_id = (id_int % 2) == 0;
                        id_int /= 2;
                        x = id_int % 128;
                        y = id_int / 128;
                        if even_id {
                            // We can write the color as is.
                            col = Color {
                                r: layer,
                                g: layer,
                                b: layer,
                                a: layer,
                            };
                        } else {
                            // We have to combine with the odd color.
                            let new_col = fragment_def_image.get_pixel(x, y).r + layer * 16.0;
                            col = Color {
                                r: new_col,
                                g: new_col,
                                b: new_col,
                                a: new_col,
                            };
                        }
                        fragment_def_image.set_pixel(x, y, col);
                    });

                    img.into_shared()
                })
                .collect();
        }

        vertex_def_image.unlock();
        let vertex_def_image = vertex_def_image.into_shared();
        fragment_def_image.unlock();
        let fragment_def_image = fragment_def_image.into_shared();

        // Create def texture.
        let vertex_def_texture = ImageTexture::new();
        vertex_def_texture.create_from_image(vertex_def_image.clone(), 0);

        // Create def layer texture.
        let fragment_def_texture = ImageTexture::new();
        fragment_def_texture.create_from_image(fragment_def_image.clone(), 0);

        let num_sprite_atlases = sprite_atlases.len();

        // Create sprite array texture.
        let sprite_array_texture = TextureArray::new();
        sprite_array_texture.create(
            sprite_atlases_size,
            sprite_atlases_size,
            num_sprite_atlases as i64,
            Image::FORMAT_RGBA8,
            0,
        );
        sprite_atlases.iter().enumerate().for_each(|(i, atlas)| {
            sprite_array_texture.set_layer_data(atlas, i64::try_from(i).unwrap());
        });

        Self {
            corrupted,
            entities_bundles: yaml_parse_result.entity_bundles,
            factions: yaml_parse_result.factions,
            mod_order,
            num_sprite_atlases,
            vertex_def_image: Some(vertex_def_image),
            vertex_def_texture: Some(vertex_def_texture),
            fragment_def_image: Some(fragment_def_image),
            fragment_def_texture: Some(fragment_def_texture),
            sprite_atlases: Some(sprite_atlases),
            sprite_array_texture: Some(sprite_array_texture),
        }
    }
}

/// Represent a mod with its version.
#[derive(Hash, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ModInfo {
    pub name: String,
    pub version: String,
}
impl Default for ModInfo {
    fn default() -> Self {
        Self {
            name: APP_NAME.to_string(),
            version: String::new(),
        }
    }
}
impl ModInfo {
    /// Return a path to the mod folder. Does not guaranty that the mod folder exist.
    /// # Examples
    /// MonInfo {name: "A super mod", version: "10.10 extra plus"}
    ///
    /// user://mods/A super mod/10_10 extra plus/
    pub fn get_path(&self) -> String {
        if self.name == APP_NAME {
            // The base mod is in res://base/
            BASE_MOD_PATH.to_string()
        } else if !self.version.is_empty() {
            // A regular mod is in user://mods/mod_name/version_name/
            format!("{}{}/{}/", MODS_PATH, self.name, self.version).replace(".", "_")
        } else {
            // A regular mod without version is in user://mods/mod_name/
            format!("{}{}/", MODS_PATH, self.name).replace(".", "_")
        }
    }
}

/// Load saved mod_order or return a default one with just APP_NAME.
fn load_mod_order(world_path: &str) -> Vec<ModInfo> {
    let mut mod_order = Vec::new();

    let file = gdnative::api::File::new();

    // Try to open saved mod_order.
    let mod_order_path = format!("{}mod_order", &world_path);
    if file.open(&mod_order_path, gdnative::api::File::READ).is_ok() {
        let mut line = file.get_csv_line(",");
        while !line.is_empty() {
            {
                let line_read = line.read();

                // Get mod name.
                if let Some(mod_name) = line_read.first() {
                    if mod_name.is_empty() {
                        break;
                    }

                    // Get mod version.
                    let mut mod_version = String::new();
                    if let Some(new_mod_version) = line_read.get(1) {
                        mod_version = new_mod_version.to_string();
                    }

                    mod_order.push(ModInfo {
                        name: mod_name.to_string(),
                        version: mod_version,
                    });
                } else {
                    break;
                }
            }
            line = file.get_csv_line(",");
        }
    } else {
        godot_error!("Could not open {}.", mod_order_path);
    }
    file.close();

    // Make sure we a have at least one mod.
    if mod_order.is_empty() {
        mod_order.push(ModInfo::default());
    }

    godot_print!("Loaded mod_order: {:?}.", &mod_order);

    mod_order
}

/// Dynamicaly load an image from disk and perform some check (size, format, etc).
/// TODO: Sprite origing.
fn load_image(path: &str) -> Option<Ref<Image, Shared>> {
    let img = Image::new();
    match img.load(path) {
        Ok(_) => {
            // Compression.
            if img.is_compressed() {
                godot_warn!("Loaded image {} is compressed. Decompressing.", path);
                if img.decompress().is_err() {
                    godot_error!("Could not decompress image {}.", path);
                    return None;
                }
            }
            // Format.
            if img.get_format().0 != Image::FORMAT_RGBA8 {
                godot_warn!("Loaded image {} as wrong format. Converting to RGBA8.", path);
                img.convert(Image::FORMAT_RGBA8);
            }
            // Crop uneeded pixels.
            let mut crop_needed = false;
            let used_rect = img.get_used_rect();
            let used_rect_int: (i64, i64, i64, i64) = unsafe {
                (
                    used_rect.position.x.to_int_unchecked(),
                    used_rect.position.y.to_int_unchecked(),
                    used_rect.size.x.to_int_unchecked(),
                    used_rect.size.y.to_int_unchecked(),
                )
            };
            let img = img.into_shared();
            if used_rect_int.0 != 0 || used_rect_int.1 != 0 {
                crop_needed = true;
                unsafe { img.assume_safe().blit_rect(img.clone(), used_rect, Vector2::ZERO) };
            }
            // Size.
            let img_safe = unsafe { img.assume_safe() };
            let too_large = img_safe.get_width() > MIN_SPRITE_ATLAS_SIZE || img_safe.get_height() > MIN_SPRITE_ATLAS_SIZE;
            if crop_needed || used_rect_int.2 != img_safe.get_width() || used_rect_int.3 != img_safe.get_height() || too_large {
                if too_large {
                    godot_warn!("Image {} is too large {:?}. Cropping.", path, img_safe.get_size());
                } else {
                    godot_print!("Image {} has unused space. Cropping.", path);
                }
                img_safe.crop(
                    used_rect_int.2.clamp(1, MIN_SPRITE_ATLAS_SIZE),
                    used_rect_int.3.clamp(0, MIN_SPRITE_ATLAS_SIZE),
                );
            }

            Some(img)
        }
        Err(err) => {
            godot_error!("Could not find image {}. {}", path, err);
            None
        }
    }
}

fn load_cached_atlas(mut def: Def) -> Result<Def, Def> {
    todo!()
}
