use common::factions::Factions;

fn main() {
    let mut f = Factions::default();
    f.update_all();

    let yaml = serde_yaml::to_string(&f).unwrap();

    println!("{}", yaml);
    // println!("{:#?}", serde_yaml::from_str::<Factions>(yaml.as_str()).unwrap());
}
