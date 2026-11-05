use sigmaos_archmanweb::*;

#[test]
fn component_is_constructible_and_runnable() {
    // Arrange
    let c = sigmaos_archmanweb::Archmanweb::new("sigmaos-archmanweb").expect("valid name");
    // Act
    let out = c.run().expect("run succeeds");
    // Assert
    assert!(out.contains("sigmaos-archmanweb"));
}
