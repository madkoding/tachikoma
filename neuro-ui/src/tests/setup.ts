import '@testing-library/jest-dom'

// Polyfills jsdom for Request
if (typeof global.Request === 'undefined') {
  global.Request = Request as unknown as typeof global.Request
}

// Mock i18next for tests
vi.mock('i18next', () => ({
  default: {
    init: vi.fn(),
    use: vi.fn(),
    t: vi.fn((key) => key),
    language: 'en',
  },
}))

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      language: 'en',
      changeLanguage: vi.fn(),
    },
  }),
  withTranslation: () => (component: any) => component,
}))
