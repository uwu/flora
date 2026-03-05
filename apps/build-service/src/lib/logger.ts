import { createConsola } from 'consola'

export const logger = createConsola({
  formatOptions: {
    colors: true,
    date: false,
    compact: false
  }
})
