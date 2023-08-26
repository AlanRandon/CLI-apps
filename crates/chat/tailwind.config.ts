import type { Config } from "tailwindcss"
import typography from "@tailwindcss/typography"

export default {
	content: ["**/*.rs", "**/*.css"],
	theme: {
		extend: {},
	},
	plugins: [typography],
} satisfies Config

