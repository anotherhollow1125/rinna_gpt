pipenv run pyinstaller --onefile --hidden-import=pytorch `
--collect-data torch `
--copy-metadata torch `
--copy-metadata tqdm `
--copy-metadata regex `
--copy-metadata requests `
--copy-metadata packaging `
--copy-metadata filelock `
--copy-metadata numpy `
--copy-metadata tokenizers `
--copy-metadata sentencepiece `
--hidden-import="sklearn.utils._cython_blas" `
--hidden-import="sklearn.neighbors.typedefs" `
--hidden-import="sklearn.neighbors.quad_tree" `
--hidden-import="sklearn.tree" `
--hidden-import="sklearn.tree._utils" `
--hidden-import="sentencepiece" `
main.py

cp .\dist\main.exe ..\src-tauri\rinna.exe