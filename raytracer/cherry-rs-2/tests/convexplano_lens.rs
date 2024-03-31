use cherry_rs_2::specs::RefractiveIndexSpec;

#[test]
fn setup() {
    let _ = RefractiveIndexSpec::TabulatedK { data: vec![[0.0, 0.0]] };
}
