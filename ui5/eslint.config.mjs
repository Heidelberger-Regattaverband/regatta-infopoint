import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import prettier from "eslint-config-prettier/flat";
import globals from "globals";
import tseslint from "typescript-eslint";

export default defineConfig(
  globalIgnores(["dist/**", "node_modules/**", "coverage/**", "**/*.tsbuildinfo"]),
  {
    files: ["webapp/**/*.{ts,js}"],
    extends: [js.configs.recommended, tseslint.configs.recommended],
    languageOptions: {
      globals: globals.browser,
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname
      }
    },
    rules: {
      // UI5/JSONModel boundaries currently use any; adopt stricter rules gradually.
      "@typescript-eslint/no-explicit-any": "off",
      "prefer-const": "warn",
      "@typescript-eslint/no-unused-vars": [
        "warn",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
          ignoreRestSiblings: true
        }
      ],
      "@typescript-eslint/no-floating-promises": "warn",
      "@typescript-eslint/no-misused-promises": "warn"
    }
  },
  {
    files: ["webapp/**/*.ts"],
    // TypeScript checks names, including UI5's ambient declarations.
    rules: { "no-undef": "off" }
  },
  // Formatting belongs to Prettier, not overlapping ESLint style rules.
  prettier
);
