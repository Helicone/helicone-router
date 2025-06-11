fn main() {
    if std::env::var("PROFILE").is_ok_and(|p| p == "release") {
        // No-op for release builds
        return;
    }
}
