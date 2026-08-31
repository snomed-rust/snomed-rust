# Third-party notices

This site depends on four **Lily Design System** npm packages (Svelte 5
editions), used under the MIT License. Vite bundles the components it
actually imports into the static output in `build/`.

- Packages:
  - https://www.npmjs.com/package/lily-design-system-svelte-headless
  - https://www.npmjs.com/package/lily-design-system-svelte-theme-picker
  - https://www.npmjs.com/package/lily-design-system-svelte-text-size-picker
  - https://www.npmjs.com/package/lily-design-system-svelte-share-picker
- Project: https://github.com/LilyDesignSystem
- Author: joel@joelparkerhenderson.com
- License: MIT

```
MIT License

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
```

Lily™ and Lily Design System™ are trademarks of their author.

Components used on this site are imported directly from these npm packages
in `src/routes/+layout.svelte` and `src/routes/+page.svelte`; see
`package.json` for pinned versions. The theme picker's two swapped
stylesheets live at `static/themes/{light,dark}.css`.
