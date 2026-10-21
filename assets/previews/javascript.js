/**
 * JSDoc comment, see {@link Counter}.
 * @param {string} path - the file to read
 * @returns {Promise<string>}
 */
import { readFile } from 'fs';

const MAX_SIZE = 1024;
let pattern = /^[a-z]+\d*$/gi;

class Counter {
  constructor(start) {
    this.count = start;
  }

  increment(step = 1) {
    this.count += step; // line comment
    return this.count;
  }
}

function greet(name) {
  /* block comment */
  const message = `Hello, ${name}!\n`;
  console.log(message, [MAX_SIZE, readFile(name)], null, undefined);
  return new Counter(0).increment();
}
