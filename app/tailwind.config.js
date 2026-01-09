/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	darkMode: 'class',
	theme: {
		extend: {
			colors: {
				// Mixer-spezifische Farben
				mixer: {
					bg: '#1a1a1a',
					surface: '#2d2d2d',
					channel: '#3d3d3d',
					fader: '#4a4a4a',
					meter: {
						green: '#22c55e',
						yellow: '#eab308',
						red: '#ef4444'
					}
				}
			},
			fontFamily: {
				mono: ['JetBrains Mono', 'Fira Code', 'monospace']
			}
		}
	},
	plugins: []
};
