debug_bin := "target/debug/ytdlrs.exe"
release_bin := "target/release/ytdlrs.exe"

alias b := build
alias re := release
alias c := clean

_default:
	@just -l

#build release
release:
	cargo b -r 
	mkdir -p release
	cp {{release_bin}} ./release

#build
build:
	cargo b

#clean
clean:
	cargo clean 
	rm -fr test

#create test directory
test: build
	rm -fr test
	mkdir test
	cp {{debug_bin}} ./test

