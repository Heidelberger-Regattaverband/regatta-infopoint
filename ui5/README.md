# UI5 development checks and formatting

Install the locked dependencies from this directory:

```bash
npm ci
```

Use a Node.js version supported by the package engines (the repository currently
uses Node.js 24). The installed typescript-eslint supports the project's
TypeScript 6 version.

## Which tool does what?

| Tool                       | Purpose                                                                |
| -------------------------- | ---------------------------------------------------------------------- |
| UI5 Linter                 | UI5-specific correctness and deprecated API checks, including XML      |
| ESLint + typescript-eslint | TypeScript/JavaScript correctness and type-aware promise checks        |
| Prettier + XML plugin      | Formatting TypeScript, XML, JSON, CSS, HTML, and project configuration |
| TypeScript compiler        | Type checking; not replaced by either linter                           |

ESLint uses the recommended JavaScript/TypeScript rules. Existing `any` usage is
allowed initially. Unused declarations, `const` suggestions, and unsafe promise handling are warnings;
prefix intentionally unused names with `_`. `eslint-config-prettier` disables
overlapping style rules, so ESLint is not a second formatter.

Prettier uses two spaces, double quotes for TypeScript, semicolons, no trailing
commas, and a 120-column target. XML uses the same 120-column target and does not
force one attribute per line. Tags exceeding that width can still wrap their
attributes onto separate lines. XML formatting preserves meaningful whitespace,
attribute order, and existing attribute quote styles. The vendored Leaflet stylesheet, generated output,
dependencies, lockfiles, and local review reports are excluded. Images, SVGs,
i18n properties, and other non-code assets are not selected by the format scripts.

## Everyday commands

| Command                | Behavior                                                        |
| ---------------------- | --------------------------------------------------------------- |
| `npm run ts-typecheck` | Check types without emitting application assets                 |
| `npm run lint`         | Run both UI5 Linter and ESLint                                  |
| `npm run lint:ui5`     | Run only UI5 Linter                                             |
| `npm run lint:ts`      | Run only ESLint; warnings do not fail the command               |
| `npm run lint:strict`  | Run both linters and fail on ESLint warnings too                |
| `npm run format:check` | Check formatting without changing files                         |
| `npm run check`        | Run type checking, both linters, then formatting checks         |
| `npm run lint:fix`     | Apply available ESLint fixes to source code; review the diff    |
| `npm run test:async`   | Run isolated async-callback regression tests                    |
| `npm run format`       | Format all selected source/configuration files; review the diff |

The setup does not bulk-format existing application sources. Initial formatting
checks may flag existing files. Run `npm run format` when ready to adopt the
formatting baseline, ideally in a separate formatting-only commit. Strict
linting fails on any warning and can prevent warnings from being reintroduced. Do not silence
promise warnings blindly with `void`: handle errors or await work where appropriate.

Async UI5 route/event callbacks use `BaseController.runAsync` to handle rejected
promises while keeping callbacks synchronous. Route arguments are read before
the callback returns; refresh cleanup remains in `finally`. The `test:async`
suite transpiles current controllers in memory and uses isolated UI5 stubs to
check rejection handling, destroyed views, route argument capture, filter order,
and busy/button cleanup. It does not build assets or make HTTP requests.

To check or format just one file, use the local executable directly:

```bash
npx --no-install eslint webapp/controller/ClubsTable.controller.ts
npx --no-install prettier --check webapp/view/ClubsTable.view.xml
npx --no-install prettier --write webapp/controller/ClubsTable.controller.ts
```

Do not append a filename to `npm run format` expecting a single-file operation:
that script already includes the entire source/configuration selection.

For CI, use `npm ci` and `npm run check` after adopting the formatting baseline.
Use `npm run lint:strict` when ready to enforce zero ESLint warnings. These checks
do not build or regenerate `dist/`.

Dependency security is a separate check: use `npm audit` (or `npm audit --omit=dev`
for production dependencies only). Do not apply `npm audit fix --force` blindly;
its suggested major-version changes can break UI5 tooling.

## VS Code

Install the recommended **ESLint** and **Prettier - Code formatter** extensions.
Recommendations are listed in the root `.vscode/extensions.json`. The XML plugin
is a local npm dependency and loads from `.prettierrc.json`.

Optional user/workspace settings for formatting on save:

```json
{
  "prettier.requireConfig": true,
  "[typescript][javascript][json][jsonc][css][html][xml]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  }
}
```

If XML is not offered as a Prettier formatting target in your extension version,
also set `"prettier.documentSelectors": ["**/ui5/**/*.xml"]`. Use the **Format
Document With...** command to select Prettier instead of another XML formatter.

The CLI is the authoritative check and works independently of editor settings.
