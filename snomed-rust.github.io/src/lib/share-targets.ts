// Share destinations for the header's SharePicker (`lily-design-system-svelte-share-picker`).
// Each `href` is a pure function of (url, title, text) — see `ShareTarget` in
// that package — so this file owns the endpoints, not the component.

import type { ShareTarget } from 'lily-design-system-svelte-share-picker';

// LinkedIn, Bluesky, and Mastodon accept only a single combined text; Reddit
// keeps url and title separate. LinkedIn's `title` query param is a
// best-effort hint — LinkedIn has favoured the page's own og:title since 2020.
const combined = (title: string, url: string): string => (title ? `${title} ${url}` : url);

export const shareTargets: ShareTarget[] = [
	{
		id: 'linkedin',
		label: 'LinkedIn',
		href: (url, title) =>
			`https://www.linkedin.com/sharing/share-offsite/?url=${encodeURIComponent(url)}&title=${encodeURIComponent(title)}`
	},
	{
		id: 'mastodon',
		label: 'Mastodon',
		// Official share widget (blog.joinmastodon.org, March 2026): a hash
		// fragment, not a query string, carrying one combined text field.
		href: (url, title) => `https://share.joinmastodon.org/#text=${encodeURIComponent(combined(title, url))}`
	},
	{
		id: 'bluesky',
		label: 'Bluesky',
		href: (url, title) => `https://bsky.app/intent/compose?text=${encodeURIComponent(combined(title, url))}`
	},
	{
		id: 'reddit',
		label: 'Reddit',
		href: (url, title) =>
			`https://www.reddit.com/submit?url=${encodeURIComponent(url)}&title=${encodeURIComponent(title)}`
	}
];
