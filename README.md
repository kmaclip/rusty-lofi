A Rust-based lo-fi music player built with WebAssembly (WASM) and Web Audio API. The goal is to generate lo-fi music in the browser with a retro-styled waveform visualization. This project is currently a work in progress—audio playback and visualization are not yet fully functional, but the core structure and synthesis logic are in place.

## Overview
rusty-lofi aims to create a browser-based lo-fi music player using Rust and WebAssembly. The project generates lo-fi beats using the Karplus-Strong synthesis algorithm for melody and chords, combined with basic oscillators for bass, kick, hi-hat, and snare drums. The audio is played through the Web Audio API, and a canvas element visualizes the audio waveform in a retro, dotted-line style.

## Features (Planned)
	Generate lo-fi music with chords, melody, bass, and drums.
	Play audio in the browser using Web Audio API.
	Visualize the audio waveform on a canvas with a retro aesthetic.
	Interactive play button to start/stop the music.

rusty-lofi/
├── assets/              # (Empty) Directory for future audio assets
├── public/              # Web files
│   ├── pkg/             # Generated WASM output (gitignored)
│   ├── index.html       # Main HTML file with canvas and play button
│   └── main.js          # JavaScript to initialize WASM and draw waveform
├── src/                 # Rust source code
│   ├── audio/           # Audio synthesis module
│   │   └── synth.rs     # Karplus-Strong and oscillator implementations
│   └── lib.rs           # Main Rust logic for audio generation and WASM bindings
├── target/              # Rust build artifacts (gitignored)
├── Cargo.toml           # Rust dependencies and project config
└── README.md            # This file

## Key Files

	src/lib.rs: Core logic for generating lo-fi music, interfacing with Web Audio API, and providing samples for visualization.
	src/audio/synth.rs: Implements the Karplus-Strong synthesis algorithm and basic oscillators for sound generation.
	public/index.html: HTML structure with a canvas for visualization and a play button.
	public/main.js: JavaScript to load the WASM module, start audio, and draw the waveform.
	Cargo.toml: Defines Rust dependencies, including wasm-bindgen, web-sys, and rand.
	
## Setup Instructions
	
	Rust: Install Rust via rustup.
	wasm-pack: Install with cargo install wasm-pack.
	Node.js: Needed for serving the project locally (install via nodejs.org).
	
	1. Build and Run:
	
		Clone the Repository
		git clone https://github.com/your-username/rusty-lofi.git
		cd rusty-lofi
	
	2. Build the WASM Module:
	
		wasm-pack build --target web --out-dir public/pkg
	
	3. Serve the Project: Use a local server to serve the public/ directory:
	
		npx http-server public
		Open http://localhost:8080 in your browser.
		
	4. Interact:
	
		Click the "Play" button to start the audio generation loop.
		Check the browser console (F12) for debug logs like:
		Initial next_start_time set to: X
		Current audio context time: Y
		Playing buffer at time: Z