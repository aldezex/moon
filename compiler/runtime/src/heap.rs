use std::collections::HashMap;

use crate::value::Value;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GcRef(pub usize);

#[derive(Debug, Clone)]
pub struct ClosureObject {
    pub func_name: String,
    pub env: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum HeapObjectKind {
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Closure(ClosureObject),
}

#[derive(Debug, Clone)]
struct HeapObject {
    marked: bool,
    kind: HeapObjectKind,
}

#[derive(Debug, Default, Clone)]
pub struct HeapStats {
    pub live_objects: usize,
    pub freed_objects: usize,
}

#[derive(Debug, Default)]
pub struct Heap {
    objects: Vec<Option<HeapObject>>,
    free_list: Vec<usize>,
}

impl Heap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> HeapStats {
        let live_objects = self.objects.iter().filter(|o| o.is_some()).count();
        HeapStats {
            live_objects,
            freed_objects: self.free_list.len(),
        }
    }

    pub fn alloc_array(&mut self, elements: Vec<Value>) -> GcRef {
        self.alloc(HeapObjectKind::Array(elements))
    }

    pub fn alloc_object(&mut self, entries: HashMap<String, Value>) -> GcRef {
        self.alloc(HeapObjectKind::Object(entries))
    }

    pub fn alloc_closure(&mut self, func_name: String, env: HashMap<String, Value>) -> GcRef {
        self.alloc(HeapObjectKind::Closure(ClosureObject { func_name, env }))
    }

    pub fn closure_func_name(&self, handle: GcRef) -> Option<&str> {
        match &self.get(handle)?.kind {
            HeapObjectKind::Closure(c) => Some(c.func_name.as_str()),
            _ => None,
        }
    }

    pub fn closure_get(&self, handle: GcRef, key: &str) -> Option<&Value> {
        match &self.get(handle)?.kind {
            HeapObjectKind::Closure(c) => c.env.get(key),
            _ => None,
        }
    }

    pub fn closure_contains(&self, handle: GcRef, key: &str) -> bool {
        self.closure_get(handle, key).is_some()
    }

    pub fn closure_set(&mut self, handle: GcRef, key: String, value: Value) -> Result<(), String> {
        let obj = self.get_mut(handle)?;
        match obj.kind {
            HeapObjectKind::Closure(ref mut c) => {
                c.env.insert(key, value);
                Ok(())
            }
            _ => Err("not a closure".to_string()),
        }
    }

    pub fn closure_env_clone(&self, handle: GcRef) -> Option<HashMap<String, Value>> {
        match &self.get(handle)?.kind {
            HeapObjectKind::Closure(c) => Some(c.env.clone()),
            _ => None,
        }
    }

    pub fn array_get(&self, handle: GcRef, idx: usize) -> Option<&Value> {
        match self.get(handle)?.kind {
            HeapObjectKind::Array(ref v) => v.get(idx),
            _ => None,
        }
    }

    pub fn array_set(&mut self, handle: GcRef, idx: usize, value: Value) -> Result<(), String> {
        let obj = self.get_mut(handle)?;
        match obj.kind {
            HeapObjectKind::Array(ref mut v) => {
                if idx >= v.len() {
                    return Err(format!("index out of bounds: {idx} (len={})", v.len()));
                }
                v[idx] = value;
                Ok(())
            }
            _ => Err("not an array".to_string()),
        }
    }

    pub fn object_get(&self, handle: GcRef, key: &str) -> Option<&Value> {
        match self.get(handle)?.kind {
            HeapObjectKind::Object(ref m) => m.get(key),
            _ => None,
        }
    }

    pub fn object_set(&mut self, handle: GcRef, key: String, value: Value) -> Result<(), String> {
        let obj = self.get_mut(handle)?;
        match obj.kind {
            HeapObjectKind::Object(ref mut m) => {
                m.insert(key, value);
                Ok(())
            }
            _ => Err("not an object".to_string()),
        }
    }

    pub fn collect_garbage(&mut self, roots: &[Value]) -> HeapStats {
        // Mark phase.
        for v in roots {
            self.mark_value(v);
        }

        // Sweep phase.
        let mut freed = 0usize;
        for (i, slot) in self.objects.iter_mut().enumerate() {
            let Some(obj) = slot.as_mut() else { continue };
            if obj.marked {
                obj.marked = false;
            } else {
                *slot = None;
                self.free_list.push(i);
                freed += 1;
            }
        }

        let live_objects = self.objects.iter().filter(|o| o.is_some()).count();
        HeapStats {
            live_objects,
            freed_objects: freed,
        }
    }

    fn alloc(&mut self, kind: HeapObjectKind) -> GcRef {
        let obj = HeapObject {
            marked: false,
            kind,
        };

        if let Some(idx) = self.free_list.pop() {
            self.objects[idx] = Some(obj);
            return GcRef(idx);
        }

        let idx = self.objects.len();
        self.objects.push(Some(obj));
        GcRef(idx)
    }

    fn get(&self, handle: GcRef) -> Option<&HeapObject> {
        self.objects.get(handle.0)?.as_ref()
    }

    fn get_mut(&mut self, handle: GcRef) -> Result<&mut HeapObject, String> {
        self.objects
            .get_mut(handle.0)
            .ok_or_else(|| "invalid handle".to_string())?
            .as_mut()
            .ok_or_else(|| "dangling handle".to_string())
    }

    fn mark_value(&mut self, value: &Value) {
        match value {
            Value::Array(h) | Value::Object(h) | Value::Closure(h) => self.mark_object(*h),
            Value::Int(_)
            | Value::Bool(_)
            | Value::String(_)
            | Value::Unit
            | Value::Function(_) => {}
        }
    }

    fn mark_object(&mut self, handle: GcRef) {
        let Some(slot) = self.objects.get_mut(handle.0) else {
            return;
        };
        let Some(obj) = slot.as_mut() else { return };
        if obj.marked {
            return;
        }
        obj.marked = true;

        match obj.kind {
            HeapObjectKind::Array(ref elems) => {
                // Clone to avoid borrowing self twice (mark is recursive).
                let elems = elems.clone();
                for v in &elems {
                    self.mark_value(v);
                }
            }
            HeapObjectKind::Object(ref map) => {
                let values: Vec<Value> = map.values().cloned().collect();
                for v in &values {
                    self.mark_value(v);
                }
            }
            HeapObjectKind::Closure(ref c) => {
                let values: Vec<Value> = c.env.values().cloned().collect();
                for v in &values {
                    self.mark_value(v);
                }
            }
        }
    }
}
