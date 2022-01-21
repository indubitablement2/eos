use common::{factions::Factions, systems::Systems};

fn main() {
    let mut f = Factions::default();
    f.update_all(&mut Systems::default());

    let yaml = serde_yaml::to_string(&f).unwrap();

    println!("{}", yaml);
    // println!("{:#?}", serde_yaml::from_str::<Factions>(yaml.as_str()).unwrap());
}
