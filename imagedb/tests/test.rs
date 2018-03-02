extern crate imagedb;

#[test]
fn it_works() {
    let foo = imagedb::InMemoryDatabase::new();
    assert_eq!(2 + 2, 4);
}