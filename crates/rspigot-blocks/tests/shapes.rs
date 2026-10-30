use rspigot_blocks::{
    Aabb, Air, Block, BlockKind, Face, FaceSupport, Glass, OakFence, OakLeaves, OakStairs,
    PerState, SHAPE_COUNT, SHAPES, STATE_COUNT, ShapeId, StateFlags, StateId, Stone, Support,
    Torch,
};

const SUPPORTS: [Support; 3] = [Support::Full, Support::Center, Support::Rigid];

const CUBE: Aabb = Aabb {
    min_x: 0.0,
    min_y: 0.0,
    min_z: 0.0,
    max_x: 1.0,
    max_y: 1.0,
    max_z: 1.0,
};

fn all_sturdy(state: StateId) -> bool {
    SUPPORTS.iter().all(|&support| {
        Face::ALL
            .iter()
            .all(|&face| state.is_face_sturdy(face, support))
    })
}

fn none_sturdy(state: StateId) -> bool {
    SUPPORTS.iter().all(|&support| {
        Face::ALL
            .iter()
            .all(|&face| !state.is_face_sturdy(face, support))
    })
}

#[test]
fn stone_is_a_sturdy_full_cube() {
    assert_eq!(Stone::DEFAULT_STATE.collision_shape(), &[CUBE]);
    assert_eq!(Stone::COLLISION, PerState::Uniform(ShapeId::FULL));
    assert_eq!(Stone::FACE_SUPPORT, PerState::Uniform(FaceSupport::ALL));
    assert!(all_sturdy(Stone::DEFAULT_STATE));
}

#[test]
fn air_has_no_shape_and_no_support() {
    assert!(StateId::AIR.collision_shape().is_empty());
    assert_eq!(Air::COLLISION, PerState::Uniform(ShapeId::EMPTY));
    assert_eq!(StateId::AIR.face_support(), FaceSupport::NONE);
    assert!(none_sturdy(StateId::AIR));
}

#[test]
fn bottom_slab_is_half_height() {
    let slab = BlockKind::OakSlab.state_with(&[("type", "bottom")]);
    let slab = slab.unwrap_or_default();
    assert_eq!(slab.property("type"), Some("bottom"));
    assert_eq!(slab.collision_shape(), &[Aabb { max_y: 0.5, ..CUBE }]);
    assert!(!slab.is_face_sturdy(Face::Up, Support::Full));
    assert!(slab.is_face_sturdy(Face::Down, Support::Full));
    let top = BlockKind::OakSlab.state_with(&[("type", "top")]);
    let top = top.unwrap_or_default();
    assert!(top.is_face_sturdy(Face::Up, Support::Full));
    assert!(!top.is_face_sturdy(Face::Down, Support::Full));
}

#[test]
fn stairs_have_two_boxes() {
    assert_eq!(OakStairs::DEFAULT_STATE.collision_shape().len(), 2);
    assert!(matches!(OakStairs::COLLISION, PerState::Varies(_)));
}

#[test]
fn fence_post_is_one_and_a_half_blocks_tall() {
    let boxes = OakFence::DEFAULT_STATE.collision_shape();
    assert!(!boxes.is_empty());
    let top = boxes.iter().map(|aabb| aabb.max_y).fold(0.0, f64::max);
    assert_eq!(top, 1.5);
}

#[test]
fn leaves_collide_fully_without_sturdy_faces() {
    let leaves = OakLeaves::DEFAULT_STATE;
    assert_eq!(leaves.collision_shape(), &[CUBE]);
    assert!(leaves.flags().contains(StateFlags::FULL_COLLISION_CUBE));
    assert!(none_sturdy(leaves));
}

#[test]
fn glass_is_a_sturdy_full_cube() {
    assert_eq!(Glass::DEFAULT_STATE.collision_shape(), &[CUBE]);
    assert!(all_sturdy(Glass::DEFAULT_STATE));
}

#[test]
fn torch_has_no_collision() {
    assert!(Torch::DEFAULT_STATE.collision_shape().is_empty());
    assert!(none_sturdy(Torch::DEFAULT_STATE));
}

#[test]
fn shape_table_is_deduplicated() {
    assert_eq!(SHAPES.len(), SHAPE_COUNT);
    assert_eq!(ShapeId::EMPTY.boxes(), &[] as &[Aabb]);
    assert_eq!(ShapeId::FULL.boxes(), &[CUBE]);
    for (index, shape) in SHAPES.iter().enumerate() {
        assert!(SHAPES[..index].iter().all(|other| other != shape));
        for aabb in *shape {
            assert!(aabb.min_x < aabb.max_x);
            assert!(aabb.min_y < aabb.max_y);
            assert!(aabb.min_z < aabb.max_z);
        }
    }
    assert!(ShapeId(SHAPE_COUNT as u16).boxes().is_empty());
}

#[test]
fn every_state_shape_resolves() {
    let mut used = vec![false; SHAPE_COUNT];
    for id in 0..STATE_COUNT as u16 {
        let state = StateId(id);
        let shape = state.collision_shape_id();
        assert!(usize::from(shape.0) < SHAPE_COUNT);
        used[usize::from(shape.0)] = true;
        assert_eq!(state.collision_shape(), shape.boxes());
        let flags = state.flags();
        assert_eq!(
            flags.contains(StateFlags::HAS_COLLISION),
            !state.collision_shape().is_empty()
        );
        if flags.contains(StateFlags::FULL_COLLISION_CUBE) {
            assert_eq!(state.collision_shape(), &[CUBE]);
        }
        let support = state.face_support();
        for face in Face::ALL {
            for support_type in SUPPORTS {
                assert_eq!(
                    state.is_face_sturdy(face, support_type),
                    support.is_sturdy(face, support_type)
                );
            }
            if support.is_sturdy(face, Support::Full) {
                assert!(support.is_sturdy(face, Support::Center));
                assert!(support.is_sturdy(face, Support::Rigid));
            }
        }
    }
    assert!(used.iter().all(|&shape| shape));
    let outside = StateId(STATE_COUNT as u16);
    assert!(outside.collision_shape().is_empty());
    assert_eq!(outside.collision_shape_id(), ShapeId::EMPTY);
    assert!(none_sturdy(outside));
}
