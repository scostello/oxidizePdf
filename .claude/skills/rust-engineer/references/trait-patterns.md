# Trait Design Patterns Reference

## Table of Contents
- [Extension Traits](#extension-traits)
- [Sealed Traits](#sealed-traits)
- [Generic Associated Types (GATs)](#generic-associated-types-gats)
- [Marker Traits](#marker-traits)
- [Trait Object Patterns](#trait-object-patterns)
- [Builder Pattern with Traits](#builder-pattern-with-traits)

## Extension Traits

Add methods to types you don't own without orphan rules issues.

```rust
// Define the extension trait
pub trait IteratorExt: Iterator {
    fn intersperse_with<F>(self, separator: F) -> IntersperseWith<Self, F>
    where
        Self: Sized,
        F: FnMut() -> Self::Item,
    {
        IntersperseWith::new(self, separator)
    }
}

// Blanket impl for all iterators
impl<I: Iterator> IteratorExt for I {}

// Usage
let items: Vec<_> = [1, 2, 3]
    .into_iter()
    .intersperse_with(|| 0)
    .collect();
// [1, 0, 2, 0, 3]
```

### Naming Convention
- `{Type}Ext` for single-type extensions (e.g., `ResultExt`, `OptionExt`)
- `{Trait}Ext` for trait extensions (e.g., `IteratorExt`, `FutureExt`)

## Sealed Traits

Prevent external implementations while exposing the trait publicly.

```rust
mod private {
    pub trait Sealed {}
}

/// Trait for internal node types. Cannot be implemented outside this crate.
pub trait Node: private::Sealed {
    fn id(&self) -> NodeId;
    fn children(&self) -> &[NodeId];
}

// Only these types can implement Node
pub struct Element { /* ... */ }
pub struct Text { /* ... */ }

impl private::Sealed for Element {}
impl private::Sealed for Text {}

impl Node for Element { /* ... */ }
impl Node for Text { /* ... */ }
```

### When to Seal
- Exhaustive pattern matching on trait implementors
- Future-proofing APIs against breaking changes
- Preventing invalid implementations that violate invariants

## Generic Associated Types (GATs)

Enable associated types with generic parameters.

```rust
trait LendingIterator {
    type Item<'a> where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;
}

// Impl that borrows from self
struct WindowsMut<'w, T> {
    slice: &'w mut [T],
    pos: usize,
    size: usize,
}

impl<'w, T> LendingIterator for WindowsMut<'w, T> {
    type Item<'a> = &'a mut [T] where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.pos + self.size > self.slice.len() {
            return None;
        }
        let window = &mut self.slice[self.pos..self.pos + self.size];
        self.pos += 1;
        Some(window)
    }
}
```

### Common GAT Use Cases
- Lending iterators (borrowing from self)
- Async traits with lifetimes
- Generic collection traits

## Marker Traits

Indicate properties without requiring methods.

```rust
/// Marker indicating a type can be safely sent to another thread.
/// Already provided by std: `Send`, `Sync`
pub unsafe trait ThreadSafe: Send + Sync {}

// Auto-implement for types that are both Send and Sync
unsafe impl<T: Send + Sync> ThreadSafe for T {}

/// Marker for types that are cheap to clone.
pub trait CheapClone: Clone {}

impl CheapClone for Arc<str> {}
impl CheapClone for Rc<str> {}
impl<T> CheapClone for &T {}
```

### Standard Marker Traits
- `Send` - Safe to transfer between threads
- `Sync` - Safe to share between threads (&T is Send)
- `Unpin` - Safe to move after pinning
- `Copy` - Bitwise copy semantics
- `Sized` - Known size at compile time (default bound)

## Trait Object Patterns

### Object-Safe Trait Design
```rust
// Object-safe: no generic methods, no Self in return position
pub trait Handler {
    fn handle(&self, request: &Request) -> Response;
    fn name(&self) -> &str;
}

// Use trait objects for runtime polymorphism
fn dispatch(handlers: &[Box<dyn Handler>], req: &Request) -> Option<Response> {
    handlers.iter()
        .find(|h| h.name() == req.handler_name)
        .map(|h| h.handle(req))
}
```

### Clone for Trait Objects
```rust
pub trait CloneBox {
    fn clone_box(&self) -> Box<dyn CloneBox>;
}

impl<T: Clone + 'static> CloneBox for T {
    fn clone_box(&self) -> Box<dyn CloneBox> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn CloneBox> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
```

## Builder Pattern with Traits

### Type-State Builder
```rust
pub struct RequestBuilder<State> {
    url: String,
    method: Method,
    headers: HeaderMap,
    body: Option<Bytes>,
    _state: PhantomData<State>,
}

// States
pub struct NoUrl;
pub struct HasUrl;
pub struct Ready;

impl RequestBuilder<NoUrl> {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            method: Method::GET,
            headers: HeaderMap::new(),
            body: None,
            _state: PhantomData,
        }
    }

    pub fn url(self, url: impl Into<String>) -> RequestBuilder<HasUrl> {
        RequestBuilder {
            url: url.into(),
            method: self.method,
            headers: self.headers,
            body: self.body,
            _state: PhantomData,
        }
    }
}

impl RequestBuilder<HasUrl> {
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.parse().unwrap(), value.parse().unwrap());
        self
    }

    pub fn body(self, body: impl Into<Bytes>) -> RequestBuilder<Ready> {
        RequestBuilder {
            url: self.url,
            method: self.method,
            headers: self.headers,
            body: Some(body.into()),
            _state: PhantomData,
        }
    }

    // Can build without body for GET/HEAD
    pub fn build(self) -> Request {
        Request { /* ... */ }
    }
}

impl RequestBuilder<Ready> {
    pub fn build(self) -> Request {
        Request { /* ... */ }
    }
}

// Usage - compile error if URL not set
let req = RequestBuilder::new()
    .url("https://api.example.com")
    .method(Method::POST)
    .body(json!({ "key": "value" }))
    .build();
```
