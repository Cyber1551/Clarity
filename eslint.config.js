import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import react from 'eslint-plugin-react'
import tseslint from 'typescript-eslint'

export default tseslint.config(
  { ignores: ['dist', 'node_modules', 'src-tauri/target'] },

  js.configs.recommended,

  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      ...tseslint.configs.recommendedTypeChecked,
      ...tseslint.configs.stylisticTypeChecked,
    ],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    plugins: {
      react,
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    settings: {
      // TODO: https://github.com/jsx-eslint/eslint-plugin-react/issues/3977
      // eslint-plugin-react's `detect` is incompatible with ESLint 10's new rule-context API.
      react: { version: '19' },
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      'react-refresh/only-export-components': [
        'warn',
        { allowConstantExport: true },
      ],
      'react/jsx-curly-brace-presence': ['warn', { props: 'never', children: 'never' }],
      'react/display-name': 'warn',
      '@typescript-eslint/consistent-type-imports': [
        'warn',
        { prefer: 'type-imports', fixStyle: 'inline-type-imports' },
      ],

      // Surface async error mishandling.
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-misused-promises': 'error',
      'react-hooks/exhaustive-deps': 'error',
      'no-console': 'error',
      'no-restricted-imports': [
        'error',
        {
          paths: [
            {
              name: '@tauri-apps/api/core',
              importNames: ['invoke'],
              message:
                'Import from src/api/* (which routes through _invoke) instead of importing invoke directly.',
            },
          ],
        },
      ],
    },
  },

  // Disable type-aware rules on JS config files (e.g. this file).
  {
    files: ['**/*.{js,mjs,cjs}'],
    extends: [tseslint.configs.disableTypeChecked],
  },

  // The logger module is the single allowed home for console.* calls and direct `invoke` imports
  // (forwarding logs to Rust must not recurse through _invoke, which itself logs via the logger).
  {
    files: ['src/utils/logger.ts'],
    rules: {
      'no-console': 'off',
      'no-restricted-imports': 'off',
    },
  },

  // The API layer is the single allowed home for direct @tauri-apps/api/core imports.
  {
    files: ['src/api/**/*.ts'],
    rules: {
      'no-restricted-imports': 'off',
    },
  },
)
