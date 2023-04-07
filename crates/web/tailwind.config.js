/** @type {import('tailwindcss').Config} */

const colors = require("tailwindcss/colors")

module.exports = {
	content: ["./**/*.{html,rs,css}"],
	theme: {
		extend: {
			colors: {
				transparent: "transparent",
				primary: colors.sky,
			},
		},
	},
	plugins: [],
}
