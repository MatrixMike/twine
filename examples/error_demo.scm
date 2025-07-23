;; Error Handling Demonstration for Twine Scheme Interpreter
;; This file shows why unbound identifier errors are essential

;; Example 1: Typo in identifier name
(define my-counter 42)
(display my-count)  ; Typo! Should be "my-counter"
;; Without error handling: undefined behavior or crash
;; With error handling: "Unbound identifier: 'my-count'. Did you mean: 'my-counter'?"

;; Example 2: Forgetting to define an identifier
(define (calculate-area width)
  (* width height))  ; Forgot to define or pass 'height'
;; Without error handling: silent failure or crash
;; With error handling: "Unbound identifier: 'height'. Make sure the identifier is defined before use"

;; Example 3: Normal binding shadowing (no warning - this is expected behavior)
(define global-binding 100)
(let ((global-binding 50))  ; This binding shadows the outer binding
  (display global-binding)) ; Uses inner binding (50) - shadowing is normal

;; Example 4: Mathematical operations with undefined identifiers
(define result (+ x y z))  ; None of these are defined
;; Without error handling: arithmetic on undefined values
;; With error handling: Clear error message for first undefined identifier

;; Example 5: Function calls with undefined identifiers
(unknown-function 1 2 3)
;; Without error handling: attempt to call undefined value
;; With error handling: "Unbound identifier: 'unknown-function'"

;; These examples show how unbound identifier errors:
;; 1. Prevent silent bugs
;; 2. Help catch typos quickly
;; 3. Provide educational feedback
;; 4. Ensure language specification compliance
;; 5. Make debugging much easier
