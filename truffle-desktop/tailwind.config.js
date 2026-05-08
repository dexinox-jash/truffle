// SPEC v2.0 SECTION 5.1.1: Darkroom Design System
// Tailwind configuration for Truffle Desktop

/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ['class'],
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      // Darkroom Color Palette
      colors: {
        // Core backgrounds
        background: '#0A0A0A',
        surface: '#141414',
        'surface-elevated': '#1A1A1A',
        'surface-hover': '#1F1F1F',
        
        // Primary accent (Amber)
        primary: {
          DEFAULT: '#FFB800',
          50: '#FFF8E6',
          100: '#FFEFCC',
          200: '#FFE099',
          300: '#FFD066',
          400: '#FFC133',
          500: '#FFB800',
          600: '#CC9300',
          700: '#996E00',
          800: '#664A00',
          900: '#332500',
        },
        
        // Secondary accent (Teal)
        secondary: {
          DEFAULT: '#00D4AA',
          50: '#E6FDF8',
          100: '#CCFBF2',
          200: '#99F7E5',
          300: '#66F3D8',
          400: '#33EFCB',
          500: '#00D4AA',
          600: '#00AA88',
          700: '#008066',
          800: '#005544',
          900: '#002B22',
        },
        
        // Text colors
        'text-primary': '#E5E5E5',
        'text-secondary': '#737373',
        'text-muted': '#525252',
        'text-disabled': '#404040',
        
        // Border colors
        border: '#262626',
        'border-hover': '#404040',
        'border-active': '#525252',
        
        // Status colors
        success: '#00D4AA',
        warning: '#FFB800',
        error: '#FF4444',
        info: '#3B82F6',
        
        // Compilation-specific
        'compile-idle': '#262626',
        'compile-active': '#FFB800',
        'compile-complete': '#00D4AA',
      },
      
      // Typography
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Menlo', 'Monaco', 'monospace'],
        serif: ['Source Serif 4', 'Georgia', 'serif'],
      },
      
      fontSize: {
        '2xs': ['0.625rem', { lineHeight: '0.875rem' }],
        'xs': ['0.75rem', { lineHeight: '1rem' }],
        'sm': ['0.875rem', { lineHeight: '1.25rem' }],
        'base': ['1rem', { lineHeight: '1.5rem' }],
        'lg': ['1.125rem', { lineHeight: '1.75rem' }],
        'xl': ['1.25rem', { lineHeight: '1.75rem' }],
        '2xl': ['1.5rem', { lineHeight: '2rem' }],
        '3xl': ['1.875rem', { lineHeight: '2.25rem' }],
      },
      
      // Spacing
      spacing: {
        '18': '4.5rem',
        '22': '5.5rem',
      },
      
      // Border radius
      borderRadius: {
        'sm': '4px',
        DEFAULT: '6px',
        'md': '8px',
        'lg': '12px',
        'xl': '16px',
      },
      
      // Animations
      animation: {
        'fade-in': 'fadeIn 200ms ease-out',
        'fade-out': 'fadeOut 200ms ease-in',
        'slide-in': 'slideIn 200ms ease-out',
        'slide-out': 'slideOut 200ms ease-in',
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'glow': 'glow 2s ease-in-out infinite alternate',
        'compile-fill': 'compileFill 2s ease-in-out',
        'shimmer': 'shimmer 2s linear infinite',
      },
      
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        fadeOut: {
          '0%': { opacity: '1' },
          '100%': { opacity: '0' },
        },
        slideIn: {
          '0%': { transform: 'translateY(-8px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        slideOut: {
          '0%': { transform: 'translateY(0)', opacity: '1' },
          '100%': { transform: 'translateY(-8px)', opacity: '0' },
        },
        glow: {
          '0%': { boxShadow: '0 0 4px rgba(255, 184, 0, 0.3)' },
          '100%': { boxShadow: '0 0 16px rgba(255, 184, 0, 0.6)' },
        },
        compileFill: {
          '0%': { width: '0%', backgroundColor: '#262626' },
          '50%': { backgroundColor: '#FFB800' },
          '100%': { width: '100%', backgroundColor: '#00D4AA' },
        },
        shimmer: {
          '0%': { backgroundPosition: '-200% 0' },
          '100%': { backgroundPosition: '200% 0' },
        },
      },
      
      // Transitions
      transitionDuration: {
        'fast': '100ms',
        DEFAULT: '200ms',
        'slow': '300ms',
      },
      
      transitionTimingFunction: {
        DEFAULT: 'ease-out',
        'bounce': 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
      },
      
      // Box shadows (minimal, flat design)
      boxShadow: {
        'focus': '0 0 0 2px rgba(255, 184, 0, 0.4)',
        'glow-sm': '0 0 8px rgba(255, 184, 0, 0.3)',
        'glow': '0 0 16px rgba(255, 184, 0, 0.4)',
        'glow-lg': '0 0 24px rgba(255, 184, 0, 0.5)',
      },
      
      // Z-index scale
      zIndex: {
        'base': '0',
        'dropdown': '100',
        'sticky': '200',
        'fixed': '300',
        'modal-backdrop': '400',
        'modal': '500',
        'popover': '600',
        'tooltip': '700',
        'command-palette': '800',
        'toast': '900',
      },
    },
  },
  plugins: [
    require('tailwindcss-animate'),
  ],
}
