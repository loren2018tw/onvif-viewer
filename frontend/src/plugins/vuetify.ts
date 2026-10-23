import 'vuetify/styles'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'

const cyberpunkColors = {
  primary: '#22D3EE',
  secondary: '#F472B6',
  accent: '#A3E635',
  error: '#FB7185',
  info: '#38BDF8',
  success: '#34D399',
  warning: '#FBBF24',
  background: '#04060D',
  surface: '#0B1120',
  'surface-bright': '#101A2E',
  'surface-light': '#16213A',
  'surface-variant': '#1B2740',
  'on-primary': '#04121A',
  'on-secondary': '#1A0612',
  'on-accent': '#101A02',
  'on-background': '#E6EDF7',
  'on-surface': '#E6EDF7',
  'on-surface-variant': '#93A4BD',
  'on-error': '#1A0508',
  'on-info': '#04121A',
  'on-success': '#04120C',
  'on-warning': '#1A1204',
}

const daylightColors = {
  primary: '#0E7490',
  secondary: '#BE185D',
  accent: '#4D7C0F',
  error: '#BE123C',
  info: '#0369A1',
  success: '#047857',
  warning: '#B45309',
  background: '#E9EFF7',
  surface: '#F8FAFC',
  'surface-bright': '#FFFFFF',
  'surface-light': '#F1F5F9',
  'surface-variant': '#DCE6F2',
  'on-primary': '#FFFFFF',
  'on-secondary': '#FFFFFF',
  'on-accent': '#FFFFFF',
  'on-background': '#0B1220',
  'on-surface': '#0B1220',
  'on-surface-variant': '#475569',
  'on-error': '#FFFFFF',
  'on-info': '#FFFFFF',
  'on-success': '#FFFFFF',
  'on-warning': '#FFFFFF',
}

export default createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: 'dark',
    themes: {
      dark: {
        dark: true,
        colors: cyberpunkColors,
        variables: {
          'high-emphasis-opacity': 0.96,
          'medium-emphasis-opacity': 0.76,
          'border-opacity': 0.22,
        },
      },
      light: {
        dark: false,
        colors: daylightColors,
        variables: {
          'high-emphasis-opacity': 0.94,
          'medium-emphasis-opacity': 0.74,
          'border-opacity': 0.24,
        },
      },
    },
  },
})
