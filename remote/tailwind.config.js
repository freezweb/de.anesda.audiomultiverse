/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			colors: {
				mixer: {
					bg: '#1a1a2e',
					surface: '#16213e',
					accent: '#0f3460',
					highlight: '#e94560'
				}
			}
		}
	},
	plugins: []
};
