import { describe, it, expect, beforeEach, afterEach } from 'vitest';

// Import the bridge (side-effect: sets window.__codemirror)
await import('../../js/codemirror-bridge.js');

describe('codemirror-bridge', () => {
  let container;

  beforeEach(() => {
    container = document.createElement('div');
    document.body.appendChild(container);
  });

  afterEach(() => {
    document.body.removeChild(container);
  });

  it('exposes window.__codemirror global', () => {
    expect(window.__codemirror).toBeDefined();
    expect(typeof window.__codemirror.create).toBe('function');
    expect(typeof window.__codemirror.destroy).toBe('function');
    expect(typeof window.__codemirror.getContent).toBe('function');
    expect(typeof window.__codemirror.setContent).toBe('function');
    expect(typeof window.__codemirror.focus).toBe('function');
    expect(typeof window.__codemirror.isDirty).toBe('function');
    expect(typeof window.__codemirror.markClean).toBe('function');
    expect(typeof window.__codemirror.onChange).toBe('function');
    expect(typeof window.__codemirror.setTheme).toBe('function');
    expect(typeof window.__codemirror.setSchema).toBe('function');
  });

  it('create returns an editor ID', () => {
    const id = window.__codemirror.create(container, { content: 'SELECT 1' });
    expect(typeof id).toBe('number');
    expect(id).toBeGreaterThan(0);
    window.__codemirror.destroy(id);
  });

  it('getContent returns initial content', () => {
    const id = window.__codemirror.create(container, { content: 'Hello World' });
    expect(window.__codemirror.getContent(id)).toBe('Hello World');
    window.__codemirror.destroy(id);
  });

  it('setContent replaces content', () => {
    const id = window.__codemirror.create(container, { content: 'original' });
    window.__codemirror.setContent(id, 'replaced');
    expect(window.__codemirror.getContent(id)).toBe('replaced');
    window.__codemirror.destroy(id);
  });

  it('getContent returns empty string for destroyed editor', () => {
    const id = window.__codemirror.create(container, { content: 'test' });
    window.__codemirror.destroy(id);
    expect(window.__codemirror.getContent(id)).toBe('');
  });

  it('isDirty tracks content changes', () => {
    const id = window.__codemirror.create(container, { content: 'clean' });
    expect(window.__codemirror.isDirty(id)).toBe(false);
    window.__codemirror.setContent(id, 'dirty');
    expect(window.__codemirror.isDirty(id)).toBe(true);
    window.__codemirror.destroy(id);
  });

  it('markClean resets dirty state', () => {
    const id = window.__codemirror.create(container, { content: 'start' });
    window.__codemirror.setContent(id, 'changed');
    expect(window.__codemirror.isDirty(id)).toBe(true);
    window.__codemirror.markClean(id);
    expect(window.__codemirror.isDirty(id)).toBe(false);
    window.__codemirror.destroy(id);
  });

  it('onChange calls callback when content changes', () => {
    const id = window.__codemirror.create(container, { content: '' });
    const changes = [];
    window.__codemirror.onChange(id, (content) => changes.push(content));
    window.__codemirror.setContent(id, 'new content');
    expect(changes).toContain('new content');
    window.__codemirror.destroy(id);
  });

  it('create with readOnly option prevents edits', () => {
    const id = window.__codemirror.create(container, { content: 'readonly', readOnly: true });
    expect(window.__codemirror.getContent(id)).toBe('readonly');
    window.__codemirror.destroy(id);
  });

  it('create with json language works', () => {
    const id = window.__codemirror.create(container, { content: '{"key": "value"}', language: 'json' });
    expect(window.__codemirror.getContent(id)).toBe('{"key": "value"}');
    window.__codemirror.destroy(id);
  });

  it('multiple editors are independent', () => {
    const container2 = document.createElement('div');
    document.body.appendChild(container2);

    const id1 = window.__codemirror.create(container, { content: 'editor 1' });
    const id2 = window.__codemirror.create(container2, { content: 'editor 2' });

    expect(id1).not.toBe(id2);
    expect(window.__codemirror.getContent(id1)).toBe('editor 1');
    expect(window.__codemirror.getContent(id2)).toBe('editor 2');

    window.__codemirror.setContent(id1, 'modified 1');
    expect(window.__codemirror.getContent(id2)).toBe('editor 2');

    window.__codemirror.destroy(id1);
    window.__codemirror.destroy(id2);
    document.body.removeChild(container2);
  });
});
