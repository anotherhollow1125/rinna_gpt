# vite.config.ts


```
  resolve: {
    alias: [{ find: "@", replacement: "/src" }],
  },
```

をdefineConfigに追加した

# tsconfig.json

```
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    }
```

をcompilerOptionsに追加した