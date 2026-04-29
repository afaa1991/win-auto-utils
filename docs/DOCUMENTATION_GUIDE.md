# Documentation Structure Guide

This document explains the documentation structure and how to maintain it.

## Directory Layout

```
win-auto-utils/
├── README.md                          # Main English README
├── docs/
│   ├── README.md                      # Documentation system entry point
│   ├── DOCUMENTATION_GUIDE.md         # This file - documentation maintenance guide
│   ├── en/                            # English documentation
│   │   ├── INDEX.md                   # English documentation index
│   │   └── modules/                   # English module docs
│   │       ├── overview.md            # Module overview
│   │       ├── memory.md
│   │       ├── memory_hook.md
│   │       ├── memory_resolver.md
│   │       ├── memory_aobscan.md
│   │       ├── script_engine.md
│   │       ├── input.md
│   │       ├── process_window.md
│   │       ├── dxgi.md
│   │       ├── template_matcher.md
│   │       └── dll_injector.md
│   │
│   └── zh/                            # Chinese documentation
│       ├── README.md                  # Chinese main README
│       ├── INDEX.md                   # Chinese documentation index
│       └── modules/                   # Chinese module docs
│           ├── overview.md
│           ├── memory.md
│           ├── memory_hook.md
│           ├── memory_resolver.md
│           ├── memory_aobscan.md
│           ├── script_engine.md
│           ├── input.md
│           ├── process_window.md
│           ├── dxgi.md
│           ├── template_matcher.md
│           └── dll_injector.md
```

## Cross-Linking Strategy

### 1. Language Switch Links

Every document should have language switch links at the top:

**English documents:**
```

```


```

**Chinese documents:**
```

```


```

### 2. Navigation Links

Each module doc should link to:
- Overview document
- Related modules
- Back to README or INDEX

### 3. Consistent Paths

Use relative paths for internal linking:
- From `docs/en/modules/memory.md` to `docs/zh/modules/memory.md`: `../../zh/modules/memory.md`
- From `docs/en/modules/memory.md` to `docs/en/INDEX.md`: `../INDEX.md`
- From `README.md` to `docs/en/INDEX.md`: `docs/en/INDEX.md`

## Documentation Standards

### File Naming
- Use lowercase with underscores: `memory_hook.md`
- Same filename for both languages
- Always use `.md` extension

### Content Structure (for module docs)

1. **Title** - Module name in respective language
2. **Language Switch** - Links to parallel document
3. **Description** - Brief overview (2-3 sentences)
4. **Feature Flag** - How to enable in Cargo.toml
5. **Quick Start** - Minimal working example
6. **Key Features** - Bullet list of capabilities
7. **Usage Examples** - 2-3 detailed examples
8. **API Reference** - Main types and functions
9. **Best Practices** - Recommended approaches
10. **Common Pitfalls** - Mistakes to avoid
11. **Performance** - Performance considerations
12. **Related Modules** - Links to related docs

### Code Examples
- Use Rust code blocks with proper syntax highlighting
- Include `use` statements for clarity
- Mark non-runnable examples with `no_run`
- Keep examples concise and focused

### Writing Style

**English:**
- Use active voice
- Be concise and clear
- Use technical terms consistently
- Follow standard technical writing conventions

**Chinese:**
- Translate accurately from English
- Maintain technical terminology
- Use simplified Chinese characters
- Keep code examples in English (variable names, comments)

## Maintenance Workflow

### Adding a New Module

1. Create parallel EN/ZH markdown files manually
2. Fill in English documentation with real content
3. Translate to Chinese (or vice versa)
4. Update `overview.md` in both languages
5. Update INDEX files if needed
6. Test all links work correctly

### Updating Existing Documentation

1. Edit English version first
2. Update Chinese version to match
3. Verify cross-links still work
4. Check for consistency in terminology
5. Update examples if API changed

### Quality Checklist

Before committing documentation changes:

- [ ] Both English and Chinese versions updated
- [ ] All cross-links work correctly
- [ ] Code examples are accurate and runnable
- [ ] No broken internal links
- [ ] Consistent formatting throughout
- [ ] Language switch links present on every page
- [ ] Module listed in overview and INDEX
- [ ] Feature flags are correct
- [ ] Examples reference actual files in `examples/` directory

## Common Issues and Solutions

### Issue: Broken Links
**Solution:** Use relative paths and test by clicking through GitHub interface

### Issue: Inconsistent Terminology
**Solution:** Maintain a glossary of key terms in both languages

### Issue: Outdated Examples
**Solution:** Regularly run examples to verify they still work

### Issue: Missing Translations
**Solution:** Always create both EN and ZH versions together

## Best Practices for Documentation

1. **Keep it DRY**: Don't repeat information available elsewhere
2. **Stay Current**: Update docs when code changes
3. **Be Practical**: Focus on real-world usage
4. **Include Context**: Explain why, not just how
5. **Show, Don't Tell**: Use examples extensively
6. **Test Everything**: Verify all code examples work
7. **Think About Users**: Write for your audience's skill level

## Contributing Guidelines

When contributing documentation:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Ensure both languages are updated
5. Test all links
6. Submit a pull request
7. Respond to review feedback

## Future Enhancements

Potential improvements to consider:

- [ ] Add automated link checking
- [ ] Generate API docs from rustdoc
- [ ] Add search functionality
- [ ] Create video tutorials
- [ ] Add interactive examples
- [ ] Support more languages (Japanese, Korean, etc.)
- [ ] Build a static documentation site

---

**Last Updated**: 2026-04-30
**Maintained By**: Community contributors
