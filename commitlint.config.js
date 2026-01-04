// commitlint configuration
// https://commitlint.js.org/

module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    // Type must be one of these
    'type-enum': [
      2,
      'always',
      [
        'feat',     // New feature
        'fix',      // Bug fix
        'docs',     // Documentation
        'style',    // Formatting, missing semicolons, etc.
        'refactor', // Code change that neither fixes a bug nor adds a feature
        'perf',     // Performance improvement
        'test',     // Adding tests
        'build',    // Build system or external dependencies
        'ci',       // CI configuration
        'chore',    // Maintenance
        'revert',   // Revert a previous commit
        'deps',     // Dependency updates
        'security', // Security fixes
        'i18n',     // Internationalization
        'a11y',     // Accessibility
      ],
    ],
    // Scope can be anything but these are suggested
    'scope-enum': [
      1, // Warning only
      'always',
      [
        'rust',
        'android',
        'ai',
        'vision',
        'stealth',
        'ui',
        'config',
        'docs',
        'ci',
        'deps',
        'i18n',
        'release',
      ],
    ],
    // Subject (description) rules - allow both lower-case and sentence-case
    'subject-case': [
      2,
      'always',
      ['lower-case', 'sentence-case'],
    ],
    'subject-empty': [2, 'never'],
    'subject-full-stop': [2, 'never', '.'],
    'subject-max-length': [2, 'always', 72],
    // Header rules
    'header-max-length': [2, 'always', 100],
    // Body rules
    'body-leading-blank': [2, 'always'],
    'body-max-line-length': [2, 'always', 100],
    // Footer rules
    'footer-leading-blank': [2, 'always'],
    'footer-max-line-length': [2, 'always', 100],
  },
  // Custom help message
  helpUrl: 'https://github.com/quinnjr/fgo-sheba/blob/main/CONTRIBUTING.md#commit-message-format',
};
