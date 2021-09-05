build-debug:
#	make build-aarch64-linux-android-debug
#	make build-armv7-linux-androideabi-debug
#	make build-i686-linux-android-debug
#	make build-x86_64-linux-android-debug
#	make build-i686-unknown-linux-gnu-debug
#	make build-x86_64-unknown-linux-gnu-debug
#	make build-x86_64-apple-darwin-debug
#	make build-aarch64-apple-ios-debug
#	make build-i686-pc-windows-gnu-debug
#	make build-x86_64-pc-windows-gnu-debug
#	make build-i686-pc-windows-msvc-debug
#	make build-x86_64-pc-windows-msvc-debug

build-release:
#	make build-aarch64-linux-android-release
#	make build-armv7-linux-androideabi-release
#	make build-i686-linux-android-release
#	make build-x86_64-linux-android-release
#	make build-i686-unknown-linux-gnu-release
#	make build-x86_64-unknown-linux-gnu-release
#	make build-x86_64-apple-darwin-release
#	make build-aarch64-apple-ios-release
#	make build-i686-pc-windows-gnu-release
#	make build-x86_64-pc-windows-gnu-release
#	make build-i686-pc-windows-msvc-release
#	make build-x86_64-pc-windows-msvc-release

export-debug:
#	make export-aarch64-linux-android-debug
#	make export-armv7-linux-androideabi-debug
#	make export-i686-linux-android-debug
#	make export-x86_64-linux-android-debug
#	make export-i686-unknown-linux-gnu-debug
#	make export-x86_64-unknown-linux-gnu-debug
#	make export-x86_64-apple-darwin-debug
#	make export-aarch64-apple-ios-debug
#	make export-i686-pc-windows-gnu-debug
#	make export-x86_64-pc-windows-gnu-debug
#	make export-i686-pc-windows-msvc-debug
#	make export-x86_64-pc-windows-msvc-debug

export-release:
#	make export-aarch64-linux-android-release
#	make export-armv7-linux-androideabi-release
#	make export-i686-linux-android-release
#	make export-x86_64-linux-android-release
#	make export-i686-unknown-linux-gnu-release
#	make export-x86_64-unknown-linux-gnu-release
#	make export-x86_64-apple-darwin-release
#	make export-aarch64-apple-ios-release
#	make export-i686-pc-windows-gnu-release
#	make export-x86_64-pc-windows-gnu-release
#	make export-i686-pc-windows-msvc-release
#	make export-x86_64-pc-windows-msvc-release

build-aarch64-linux-android-debug:
	cargo build --target aarch64-linux-android 
	mv -b ./target/aarch64-linux-android/debug/*.so ./lib/aarch64-linux-android

export-aarch64-linux-android-debug: clean build-aarch64-linux-android-debug
	cd godot/ ; godot --export-debug "Android.aarch64-linux-android.debug" ../bin/aarch64-linux-android/chaos-cascade.debug.aarch64-linux-android.apk

build-aarch64-linux-android-release:
	cargo build --target aarch64-linux-android --release
	mv -b ./target/aarch64-linux-android/release/*.so ./lib/aarch64-linux-android

export-aarch64-linux-android-release: clean build-aarch64-linux-android-release
	cd godot/ ; godot --export "Android.aarch64-linux-android.release" ../bin/aarch64-linux-android/chaos-cascade.release.aarch64-linux-android.apk

build-armv7-linux-androideabi-debug:
	cargo build --target armv7-linux-androideabi 
	mv -b ./target/armv7-linux-androideabi/debug/*.so ./lib/armv7-linux-androideabi

export-armv7-linux-androideabi-debug: clean build-armv7-linux-androideabi-debug
	cd godot/ ; godot --export-debug "Android.armv7-linux-androideabi.debug" ../bin/armv7-linux-androideabi/chaos-cascade.debug.armv7-linux-androideabi.apk

build-armv7-linux-androideabi-release:
	cargo build --target armv7-linux-androideabi --release
	mv -b ./target/armv7-linux-androideabi/release/*.so ./lib/armv7-linux-androideabi

export-armv7-linux-androideabi-release: clean build-armv7-linux-androideabi-release
	cd godot/ ; godot --export "Android.armv7-linux-androideabi.release" ../bin/armv7-linux-androideabi/chaos-cascade.release.armv7-linux-androideabi.apk

build-i686-linux-android-debug:
	cargo build --target i686-linux-android 
	mv -b ./target/i686-linux-android/debug/*.so ./lib/i686-linux-android

export-i686-linux-android-debug: clean build-i686-linux-android-debug
	cd godot/ ; godot --export-debug "Android.i686-linux-android.debug" ../bin/i686-linux-android/chaos-cascade.debug.i686-linux-android.apk

build-i686-linux-android-release:
	cargo build --target i686-linux-android --release
	mv -b ./target/i686-linux-android/release/*.so ./lib/i686-linux-android

export-i686-linux-android-release: clean build-i686-linux-android-release
	cd godot/ ; godot --export "Android.i686-linux-android.release" ../bin/i686-linux-android/chaos-cascade.release.i686-linux-android.apk

build-x86_64-linux-android-debug:
	cargo build --target x86_64-linux-android 
	mv -b ./target/x86_64-linux-android/debug/*.so ./lib/x86_64-linux-android

export-x86_64-linux-android-debug: clean build-x86_64-linux-android-debug
	cd godot/ ; godot --export-debug "Android.x86_64-linux-android.debug" ../bin/x86_64-linux-android/chaos-cascade.debug.x86_64-linux-android.apk

build-x86_64-linux-android-release:
	cargo build --target x86_64-linux-android --release
	mv -b ./target/x86_64-linux-android/release/*.so ./lib/x86_64-linux-android

export-x86_64-linux-android-release: clean build-x86_64-linux-android-release
	cd godot/ ; godot --export "Android.x86_64-linux-android.release" ../bin/x86_64-linux-android/chaos-cascade.release.x86_64-linux-android.apk

build-i686-unknown-linux-gnu-debug:
	cargo build --target i686-unknown-linux-gnu 
	mv -b ./target/i686-unknown-linux-gnu/debug/*.so ./lib/i686-unknown-linux-gnu

export-i686-unknown-linux-gnu-debug: clean build-i686-unknown-linux-gnu-debug
	cd godot/ ; godot --export-debug "Linux/X11.i686-unknown-linux-gnu.debug" ../bin/i686-unknown-linux-gnu/chaos-cascade.debug.i686-unknown-linux-gnu

build-i686-unknown-linux-gnu-release:
	cargo build --target i686-unknown-linux-gnu --release
	mv -b ./target/i686-unknown-linux-gnu/release/*.so ./lib/i686-unknown-linux-gnu

export-i686-unknown-linux-gnu-release: clean build-i686-unknown-linux-gnu-release
	cd godot/ ; godot --export "Linux/X11.i686-unknown-linux-gnu.release" ../bin/i686-unknown-linux-gnu/chaos-cascade.release.i686-unknown-linux-gnu

build-x86_64-unknown-linux-gnu-debug:
	cargo build --target x86_64-unknown-linux-gnu 
	mv -b ./target/x86_64-unknown-linux-gnu/debug/*.so ./lib/x86_64-unknown-linux-gnu

export-x86_64-unknown-linux-gnu-debug: clean build-x86_64-unknown-linux-gnu-debug
	cd godot/ ; godot --export-debug "Linux/X11.x86_64-unknown-linux-gnu.debug" ../bin/x86_64-unknown-linux-gnu/chaos-cascade.debug.x86_64-unknown-linux-gnu

build-x86_64-unknown-linux-gnu-release:
	cargo build --target x86_64-unknown-linux-gnu --release
	mv -b ./target/x86_64-unknown-linux-gnu/release/*.so ./lib/x86_64-unknown-linux-gnu

export-x86_64-unknown-linux-gnu-release: clean build-x86_64-unknown-linux-gnu-release
	cd godot/ ; godot --export "Linux/X11.x86_64-unknown-linux-gnu.release" ../bin/x86_64-unknown-linux-gnu/chaos-cascade.release.x86_64-unknown-linux-gnu

build-x86_64-apple-darwin-debug:
	cargo build --target x86_64-apple-darwin 
	mv -b ./target/x86_64-apple-darwin/debug/*.dylib ./lib/x86_64-apple-darwin

export-x86_64-apple-darwin-debug: clean build-x86_64-apple-darwin-debug
	cd godot/ ; godot --export-debug "Mac OSX.x86_64-apple-darwin.debug" ../bin/x86_64-apple-darwin/chaos-cascade.debug.x86_64-apple-darwin

build-x86_64-apple-darwin-release:
	cargo build --target x86_64-apple-darwin --release
	mv -b ./target/x86_64-apple-darwin/release/*.dylib ./lib/x86_64-apple-darwin

export-x86_64-apple-darwin-release: clean build-x86_64-apple-darwin-release
	cd godot/ ; godot --export "Mac OSX.x86_64-apple-darwin.release" ../bin/x86_64-apple-darwin/chaos-cascade.release.x86_64-apple-darwin

build-aarch64-apple-ios-debug:
	cargo build --target aarch64-apple-ios 
	mv -b ./target/aarch64-apple-ios/debug/*.a ./lib/aarch64-apple-ios

export-aarch64-apple-ios-debug: clean build-aarch64-apple-ios-debug
	cd godot/ ; godot --export-debug "iOS.aarch64-apple-ios.debug" ../bin/aarch64-apple-ios/chaos-cascade.debug.aarch64-apple-ios.ipa

build-aarch64-apple-ios-release:
	cargo build --target aarch64-apple-ios --release
	mv -b ./target/aarch64-apple-ios/release/*.a ./lib/aarch64-apple-ios

export-aarch64-apple-ios-release: clean build-aarch64-apple-ios-release
	cd godot/ ; godot --export "iOS.aarch64-apple-ios.release" ../bin/aarch64-apple-ios/chaos-cascade.release.aarch64-apple-ios.ipa

build-i686-pc-windows-gnu-debug:
	cargo build --target i686-pc-windows-gnu 
	mv -b ./target/i686-pc-windows-gnu/debug/*.dll ./lib/i686-pc-windows-gnu

export-i686-pc-windows-gnu-debug: clean build-i686-pc-windows-gnu-debug
	cd godot/ ; godot --export-debug "Windows Desktop.i686-pc-windows-gnu.debug" ../bin/i686-pc-windows-gnu/chaos-cascade.debug.i686-pc-windows-gnu.exe

build-i686-pc-windows-gnu-release:
	cargo build --target i686-pc-windows-gnu --release
	mv -b ./target/i686-pc-windows-gnu/release/*.dll ./lib/i686-pc-windows-gnu

export-i686-pc-windows-gnu-release: clean build-i686-pc-windows-gnu-release
	cd godot/ ; godot --export "Windows Desktop.i686-pc-windows-gnu.release" ../bin/i686-pc-windows-gnu/chaos-cascade.release.i686-pc-windows-gnu.exe

build-x86_64-pc-windows-gnu-debug:
	cargo build --target x86_64-pc-windows-gnu 
	mv -b ./target/x86_64-pc-windows-gnu/debug/*.dll ./lib/x86_64-pc-windows-gnu

export-x86_64-pc-windows-gnu-debug: clean build-x86_64-pc-windows-gnu-debug
	cd godot/ ; godot --export-debug "Windows Desktop.x86_64-pc-windows-gnu.debug" ../bin/x86_64-pc-windows-gnu/chaos-cascade.debug.x86_64-pc-windows-gnu.exe

build-x86_64-pc-windows-gnu-release:
	cargo build --target x86_64-pc-windows-gnu --release
	mv -b ./target/x86_64-pc-windows-gnu/release/*.dll ./lib/x86_64-pc-windows-gnu

export-x86_64-pc-windows-gnu-release: clean build-x86_64-pc-windows-gnu-release
	cd godot/ ; godot --export "Windows Desktop.x86_64-pc-windows-gnu.release" ../bin/x86_64-pc-windows-gnu/chaos-cascade.release.x86_64-pc-windows-gnu.exe

build-i686-pc-windows-msvc-debug:
	cargo build --target i686-pc-windows-msvc 
	mv -b ./target/i686-pc-windows-msvc/debug/*.dll ./lib/i686-pc-windows-msvc

export-i686-pc-windows-msvc-debug: clean build-i686-pc-windows-msvc-debug
	cd godot/ ; godot --export-debug "Windows Desktop.i686-pc-windows-msvc.debug" ../bin/i686-pc-windows-msvc/chaos-cascade.debug.i686-pc-windows-msvc.exe

build-i686-pc-windows-msvc-release:
	cargo build --target i686-pc-windows-msvc --release
	mv -b ./target/i686-pc-windows-msvc/release/*.dll ./lib/i686-pc-windows-msvc

export-i686-pc-windows-msvc-release: clean build-i686-pc-windows-msvc-release
	cd godot/ ; godot --export "Windows Desktop.i686-pc-windows-msvc.release" ../bin/i686-pc-windows-msvc/chaos-cascade.release.i686-pc-windows-msvc.exe

build-x86_64-pc-windows-msvc-debug:
	cargo build --target x86_64-pc-windows-msvc 
	mv -b ./target/x86_64-pc-windows-msvc/debug/*.dll ./lib/x86_64-pc-windows-msvc

export-x86_64-pc-windows-msvc-debug: clean build-x86_64-pc-windows-msvc-debug
	cd godot/ ; godot --export-debug "Windows Desktop.x86_64-pc-windows-msvc.debug" ../bin/x86_64-pc-windows-msvc/chaos-cascade.debug.x86_64-pc-windows-msvc.exe

build-x86_64-pc-windows-msvc-release:
	cargo build --target x86_64-pc-windows-msvc --release
	mv -b ./target/x86_64-pc-windows-msvc/release/*.dll ./lib/x86_64-pc-windows-msvc

export-x86_64-pc-windows-msvc-release: clean build-x86_64-pc-windows-msvc-release
	cd godot/ ; godot --export "Windows Desktop.x86_64-pc-windows-msvc.release" ../bin/x86_64-pc-windows-msvc/chaos-cascade.release.x86_64-pc-windows-msvc.exe

audit:
	cargo-audit audit

check: clean
	cargo check

clean:
	cargo clean

create-debug-keystore:
	keytool -keyalg RSA -genkeypair -alias androiddebugkey -keypass android -keystore chaos-cascade.debug.keystore -storepass android -dname "CN=Android Debug,O=Android,C=US" -validity 9999 -deststoretype pkcs12

create-release-keystore:
	keytool -v -genkey -v -keystore chaos-cascade.release.keystore -alias chaos-cascade -keyalg RSA -validity 10000

doc: clean
	cargo doc --no-deps --open -v

edit:
#	${EDITOR} rust/src/lib.rs &
	godot --path godot/ -e &

run:
	make build-x86_64-unknown-linux-gnu-debug
	godot --path godot/ -d

shell:
	nix-shell --pure

test: clean
	cargo test
