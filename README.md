# embed_lisp
Rustで実装したLispインタプリタです。

Rust側のオブジェクトをLispから操作できるように設計しています。

## 主な機能
### 特殊フォーム
- define, lambda, quote, let, begin, define-macro など

### 組み込み関数
- 四則演算: +, -, *, /
- 比較: =, <, >
- リスト操作: car, cdr, cons, list
- 文字列操作: str::index-of, str::contains?, str::to-upper など
 

```lisp
(define square (lambda (x) (* x x)))
;; => Lambda { (x) -> (* x x) }
(square 5)
;; => 25
```
