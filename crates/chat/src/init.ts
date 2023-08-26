import Alpine from "alpinejs"
import "htmx.org"

declare global {
	interface Window {
		Alpine: typeof Alpine
	}
}

window.Alpine = Alpine

Alpine.start()
