import { defineConfig } from 'vite-plus'

const ignorePatterns = [
  '**/node_modules',
  '**/*-lock.json',
  'pnpm-lock.yaml',
  '**/bun.lock',
  '**/submodules/**',
  '**/dist',
  '**/trash',
  '**/target',
  '**/refs/**',
  'pnpm-lock.yaml'
]

export default defineConfig({
  run: {
    cache: {
      scripts: true,
      tasks: true
    },
    tasks: {
      build: {
        command: 'vp run -r build',
        env: ['NODE_ENV', 'VITE_*']
      },
      'apps:build': {
        command: [
          'vp run',
          '--filter "@flora-internal/build-service"',
          '--filter "@flora-internal/frontend"',
          '--filter "@flora-internal/uwu.network"',
          '--filter "@flora-internal/www"',
          'build'
        ].join(' '),
        dependsOn: ['@uwu/flora-api-client#build', '@uwu/flora-cli#build', '@uwu/flora-sdk#build'],
        env: ['NODE_ENV', 'VITE_*']
      },
      test: {
        command: 'vp run -r test',
        env: ['NODE_ENV', 'VITE_*']
      },
      typecheck: {
        command: 'vp run -r typecheck'
      },
      lint: {
        command: 'vp run -r lint'
      }
    }
  },
  staged: {
    '*': 'vp check --fix',
    '*.rs': 'cargo fmt'
  },
  fmt: {
    quoteProps: 'preserve',
    printWidth: 100,
    singleQuote: true,
    semi: false,
    trailingComma: 'none',
    tabWidth: 2,
    ignorePatterns,
    jsxSingleQuote: true
  },
  /**
  lint: {
    "plugins": [
    "eslint",
    "typescript",
    "unicorn",
    "react",
    "react-perf",
    "oxc",
    "jsx-a11y",
    "promise"
  ],
  "categories": {},
  "rules": {},
  "options": {
    "typeAware": true,
    "typeCheck": true
  },
  "settings": {
    "jsx-a11y": {
      "components": {},
      "attributes": {}
    },
    "next": {
      "rootDir": []
    },
    "react": {
      "formComponents": [],
      "linkComponents": [],
      "componentWrapperFunctions": []
    },
    "jsdoc": {
      "ignorePrivate": false,
      "ignoreInternal": false,
      "ignoreReplacesDocs": true,
      "overrideReplacesDocs": true,
      "augmentsExtendsReplacesDocs": false,
      "implementsReplacesDocs": false,
      "exemptDestructuredRootsFromChecks": false,
      "tagNamePreference": {}
    },
    "vitest": {
      "typecheck": true
    }
  },
  "env": {
    "builtin": true
  },
  "globals": {},
    ignorePatterns,
  },
  **/
  test: {}
})
