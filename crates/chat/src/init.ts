import "htmx.org"
import Alpine from "alpinejs"
import { intlFormatDistance } from "date-fns"

declare global {
	interface Window {
		Alpine: typeof Alpine
	}
}

window.Alpine = Alpine

Alpine.directive("show-time-since", (el, { expression }) => {
	const seconds = +expression
	const now = Date.now()
	el.textContent = intlFormatDistance(now - 1000 * seconds, now)
})

Alpine.start()
