// Web entry point for WGPU Canvas Editor
import init, { run } from './pkg/wgpu_canvas_editor.js';

async function main() {
    try {
        // Check for WebGPU support
        if (!navigator.gpu) {
            throw new Error('WebGPU is not supported in this browser. Please use Chrome 113+, Firefox 121+, or Safari 18+.');
        }

        console.log('Initializing WASM module...');
        await init();

        console.log('Starting application...');
        document.getElementById('loading').style.display = 'none';

        await run();
    } catch (error) {
        console.error('Failed to initialize:', error);
        document.getElementById('loading').innerHTML = `
            <div class="error">
                <h3>Error</h3>
                <p>${error.message}</p>
                <p>Please check the console for more details.</p>
            </div>
        `;
    }
}

main();
