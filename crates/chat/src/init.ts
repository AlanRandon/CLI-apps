import "htmx.org"
import Alpine from "alpinejs"
import moment from "moment";

declare global {
	interface Window {
		Alpine: typeof Alpine
	}
}

window.Alpine = Alpine

Alpine.directive("show-time-since", (el, { expression }) => {
	el.textContent = moment().subtract(+expression, "seconds").fromNow()
})

Alpine.start()
