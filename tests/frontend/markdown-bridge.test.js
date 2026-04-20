import { describe, it, expect } from 'vitest';

// Import the bridge (side-effect: sets window.__markdown)
await import('../../js/markdown-bridge.js');

describe('markdown-bridge', () => {
  it('exposes window.__markdown global', () => {
    expect(window.__markdown).toBeDefined();
    expect(typeof window.__markdown.render).toBe('function');
  });

  it('renders basic markdown to HTML', () => {
    const html = window.__markdown.render('Hello **world**');
    expect(html).toContain('<strong>world</strong>');
  });

  it('renders headings', () => {
    const html = window.__markdown.render('# Title\n## Subtitle');
    expect(html).toContain('<h1');
    expect(html).toContain('Title');
    expect(html).toContain('<h2');
    expect(html).toContain('Subtitle');
  });

  it('renders inline code', () => {
    const html = window.__markdown.render('Use `SELECT *` here');
    expect(html).toContain('<code>SELECT *</code>');
  });

  it('renders code blocks with language class', () => {
    const md = '```sql\nSELECT * FROM users;\n```';
    const html = window.__markdown.render(md);
    expect(html).toContain('<code');
    expect(html).toContain('SELECT');
    expect(html).toContain('language-sql');
  });

  it('renders links', () => {
    const html = window.__markdown.render('[click](https://example.com)');
    expect(html).toContain('<a');
    expect(html).toContain('href="https://example.com"');
    expect(html).toContain('click');
  });

  it('renders lists', () => {
    const html = window.__markdown.render('- item 1\n- item 2\n- item 3');
    expect(html).toContain('<ul>');
    expect(html).toContain('<li>');
    expect(html).toContain('item 1');
  });

  it('handles empty string', () => {
    const html = window.__markdown.render('');
    expect(html).toBe('');
  });

  it('handles pure code block', () => {
    const md = '```\nno language specified\n```';
    const html = window.__markdown.render(md);
    expect(html).toContain('<code');
    expect(html).toContain('no language specified');
  });

  it('handles nested formatting', () => {
    const html = window.__markdown.render('***bold and italic***');
    expect(html).toContain('<strong>');
    expect(html).toContain('<em>');
  });

  it('renders SQL code block with language class', () => {
    const md = '```sql\nCREATE TABLE users (id INT PRIMARY KEY);\n```';
    const html = window.__markdown.render(md);
    expect(html).toContain('language-sql');
    expect(html).toContain('CREATE');
  });

  it('renders JSON code block with language class', () => {
    const md = '```json\n{"key": "value", "num": 42}\n```';
    const html = window.__markdown.render(md);
    expect(html).toContain('language-json');
  });

  it('renders GFM tables', () => {
    const md = '| Col1 | Col2 |\n|------|------|\n| a | b |';
    const html = window.__markdown.render(md);
    expect(html).toContain('<table>');
    expect(html).toContain('<th>');
    expect(html).toContain('Col1');
  });

  it('handles line breaks (breaks: true)', () => {
    const html = window.__markdown.render('line 1\nline 2');
    expect(html).toContain('<br');
  });
});
