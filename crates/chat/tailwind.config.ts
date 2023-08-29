import type { Config } from "tailwindcss"
import typography from "@tailwindcss/typography"
import forms from "@tailwindcss/forms"
import colors from "tailwindcss/colors"
import plugin from "tailwindcss/plugin"

export default {
	content: ["**/*.rs", "**/*.css"],
	theme: {
		extend: {
			colors: {
				gray: colors.slate,
			},
		}
	},
	plugins: [typography, forms, plugin(({ addBase }) => {
		addBase({
			["[type='text'], input:where(:not([type])), [type='email'], [type='url'], [type='password'], [type='number'], [type='date'], [type='datetime-local'], [type='month'], [type='search'], [type='tel'], [type='time'], [type='week'], [multiple], textarea, select,"]: {
				"&:focus": {
					"--tw-ring-color": colors.orange[600],
					"border-color": colors.orange[600]
				}
			}
		})
	})],
} satisfies Config

