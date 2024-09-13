cargo fmt -- --config-path rustfmt.toml
cargo clippy --fix --allow-dirty
cargo build --features="build2d,test" --no-default-features
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    echo "Running on macOS"
    rm bin2d/addons/godot-rapier2d/bin/libgodot_rapier.macos.framework/libgodot_rapier.macos.dylib
    cp target/debug/libgodot_rapier.dylib bin2d/addons/godot-rapier2d/bin/libgodot_rapier.macos.framework/libgodot_rapier.macos.dylib
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    echo "Running on Linux"
    rm bin2d/addons/godot-rapier2d/bin/libgodot_rapier.linux.so
    cp target/debug/libgodot_rapier.so bin2d/addons/godot-rapier2d/bin/libgodot_rapier.linux.so
else
    echo "Unsupported OS: $OSTYPE"
    exit 1
fi