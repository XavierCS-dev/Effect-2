## Next Steps (Phase 1 - Renderer)
- [x] Remove EntityGroup2D - don't want it to take entity owernship
- [x] Find different way to pass refs to entities grouped by layer to render function
- [ ] Render function - iterate through layers to draw, call methods to get buffers
- [x] Layers - Create and write to buffers when needed.
- [x] TextureAtlas2D - Complete atlas, to expand across width and height, and limit self to 8096x8096
- [x] Create descriptors for entity
- [ ] Create Transform 2D maths stuff for entity (later)
- [ ] Create buffers in shader
- [x] Add buffer layouts to pipeline
- [ ] Add bind group layouts to pipeline (Mainly for entity)
- [ ] Implement Entity2D::new()
- [ ] Switch HashMap to BTreeMao where it makes sense to do so
