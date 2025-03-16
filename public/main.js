import init, { LoFiPlayer } from './pkg/rusty_lofi.js';

async function run() {
    await init();
    const player = new LoFiPlayer();
    const canvas = document.getElementById('waveCanvas');
    const ctx = canvas.getContext('2d');
    const playButton = document.getElementById('playButton');

    const width = canvas.width;
    const height = canvas.height;
    const samples = new Array(width).fill(height / 2);
    let index = 0;
    let isPlaying = false;

    playButton.addEventListener('click', () => {
        if (!isPlaying) {
            player.start_audio().catch(console.error);
            isPlaying = true;
            playButton.textContent = "Pause";
        } else {
            // You'd need to add a pause method to your Rust code
            // player.pause_audio().catch(console.error);
            // isPlaying = false;
            // playButton.textContent = "Play";
        }
    });

    function draw() {
        const sampleValue = player.get_sample();
        if (sampleValue !== undefined && sampleValue !== null) {
            samples[index] = sampleValue * height / 2 + height / 2;
            index = (index + 1) % width;
        }

        ctx.clearRect(0, 0, width, height);
        ctx.fillStyle = '#2a1b3d';
        ctx.fillRect(0, 0, width, height);
        
        ctx.beginPath();
        ctx.strokeStyle = '#fff';
        ctx.lineWidth = 2;
        
        for (let i = 0; i < width - 1; i++) {
            const x0 = i;
            const y0 = samples[(index + i) % width];
            const x1 = i + 1;
            const y1 = samples[(index + i + 1) % width];
            
            if (i === 0) {
                ctx.moveTo(x0, y0);
            } else {
                ctx.lineTo(x0, y0);
            }
        }
        ctx.stroke();
        
        requestAnimationFrame(draw);
    }

    draw();
}

run().catch(console.error);