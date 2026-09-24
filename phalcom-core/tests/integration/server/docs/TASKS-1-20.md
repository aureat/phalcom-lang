# Phalcom typing golden — tasks 1–20

## Checkpoint 1: domain and generic infrastructure

Tasks 1–10 establish:

1. package shell and executable boundary,
2. primitive domain model,
3. `Order<S: OrderState>` static typestate,
4. executable `Draft -> Submitted -> Paid` transitions,
5. HTTP `Method` / `Status` static domains,
6. parameterized `Retry` values,
7. `Effect<F>` with `F: Type -> Type`,
8. deterministic `TestFx<T>`,
9. associated-type capability interfaces,
10. deterministic fixture implementations.

## Checkpoint 2: structural pipeline and checkout application

Tasks 11–20 establish:

11. open-row request middleware with `**R` and ignored `...`,
12. middleware composition and inferred enriched request context,
13. result-indexed `Command<R>`,
14. GADT-style `CommandExecutor<TestFx>` refinement,
15. statically typed `CheckoutServices`,
16. structural-input `checkoutHandler`,
17. ownership validation and `Draft -> Submitted`,
18. effectful payment authorization through `PaymentGateway<TestFx, Count(3)>`,
19. `Submitted -> Paid`,
20. persistence through `Command::SavePaid` with `Order<Paid>` preserved.

The original Tasks 1–20 checkpoint ended immediately after persistence. In the current Tasks 1–30 package, the shared checkout implementation has been advanced through Task 21 and now publishes `OrderPaid` before constructing the response; the preceding type flow remains unchanged.
