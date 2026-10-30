import com.google.gson.stream.JsonWriter;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.server.RegistryLayer;
import net.minecraft.server.packs.PackType;
import net.minecraft.server.packs.repository.ServerPacksSource;
import net.minecraft.server.packs.resources.MultiPackResourceManager;
import net.minecraft.tags.BlockTags;
import net.minecraft.tags.TagLoader;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.EntityTypes;
import net.minecraft.world.item.BlockItem;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.StandingAndWallBlockItem;
import net.minecraft.world.level.EmptyBlockGetter;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.SupportType;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.Property;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.material.Fluids;
import net.minecraft.world.level.pathfinder.PathComputationType;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.shapes.VoxelShape;

public final class BlockDump {
    private static final Direction[] FACES = {Direction.DOWN, Direction.UP, Direction.NORTH, Direction.SOUTH, Direction.WEST, Direction.EAST};
    private static final String[] SPAWN_SURFACES = {"default", "never", "always", "ocelot_or_parrot", "polar_bear", "fire_immune"};

    private BlockDump() {
    }

    public static void main(String[] args) throws IOException {
        if (args.length != 1) {
            System.err.println("usage: BlockDump <output.json>");
            System.exit(2);
        }
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        bindTags();
        Path out = Path.of(args[0]).toAbsolutePath();
        Files.createDirectories(out.getParent());
        Path tmp = out.resolveSibling(out.getFileName() + ".tmp");
        try (JsonWriter json = new JsonWriter(Files.newBufferedWriter(tmp, StandardCharsets.UTF_8))) {
            json.setIndent(" ");
            json.beginObject();
            json.name("version").value(SharedConstants.getCurrentVersion().name());
            Map<List<Double>, Integer> shapes = new LinkedHashMap<>();
            json.name("blocks");
            writeBlocks(json, shapes);
            json.name("collision_shapes");
            writeShapes(json, shapes);
            json.name("block_items");
            writeBlockItems(json);
            json.endObject();
        }
        Files.move(tmp, out, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
        System.err.println("wrote " + out);
    }

    private static void bindTags() {
        try (MultiPackResourceManager resources = new MultiPackResourceManager(PackType.SERVER_DATA, List.of(ServerPacksSource.createVanillaPackSource().fullResources()))) {
            List<Registry.PendingTags<?>> pending = TagLoader.loadTagsForExistingRegistries(resources, RegistryLayer.createRegistryAccess().getLayer(RegistryLayer.STATIC));
            pending.forEach(Registry.PendingTags::apply);
        }
        if (!Blocks.STONE.defaultBlockState().is(BlockTags.BLOCKS_MOTION_IN_HEIGHTMAP)) {
            throw new IllegalStateException("block tags not bound");
        }
    }

    private static void writeBlocks(JsonWriter json, Map<List<Double>, Integer> shapes) throws IOException {
        json.beginArray();
        int expectedState = 0;
        for (int id = 0; id < BuiltInRegistries.BLOCK.size(); id++) {
            Block block = BuiltInRegistries.BLOCK.byId(id);
            if (BuiltInRegistries.BLOCK.getId(block) != id) {
                throw new IllegalStateException("registry id mismatch at " + id);
            }
            BlockState defaultState = block.defaultBlockState();
            json.beginObject();
            json.name("name").value(BuiltInRegistries.BLOCK.getKey(block).toString());
            json.name("id").value(id);
            json.name("class").value(block.getClass().getSimpleName());
            Item item = block.asItem();
            json.name("item");
            if (item == Items.AIR) {
                json.nullValue();
            } else {
                json.value(BuiltInRegistries.ITEM.getId(item));
            }
            json.name("item_block");
            if (item instanceof BlockItem blockItem) {
                json.value(BuiltInRegistries.BLOCK.getId(blockItem.getBlock()));
            } else {
                json.nullValue();
            }
            json.name("wall_variant");
            if (item instanceof StandingAndWallBlockItem standing) {
                json.value(BuiltInRegistries.BLOCK.getId(wallBlock(standing)));
            } else {
                json.nullValue();
            }
            json.name("destroy_time").value(Float.valueOf(block.defaultDestroyTime()));
            json.name("explosion_resistance").value(Float.valueOf(block.getExplosionResistance()));
            json.name("friction").value(Float.valueOf(block.getFriction()));
            json.name("speed_factor").value(Float.valueOf(block.getSpeedFactor()));
            json.name("jump_factor").value(Float.valueOf(block.getJumpFactor()));
            json.name("bounce_restitution").value(Float.valueOf(block.getBounceRestitution()));
            json.name("fall_distance_reduction").value(Float.valueOf(block.getFallDistanceReduction()));
            json.name("requires_correct_tool_for_drops").value(defaultState.requiresCorrectToolForDrops());
            json.name("ignited_by_lava").value(defaultState.ignitedByLava());
            json.name("default_state").value(Block.getId(defaultState));
            json.name("spawn_surface").value(spawnSurface(block));
            json.name("properties").beginArray();
            for (Property<?> property : block.getStateDefinition().getProperties()) {
                json.beginObject();
                json.name("name").value(property.getName());
                json.name("values").beginArray();
                for (String value : valueNames(property)) {
                    json.value(value);
                }
                json.endArray();
                json.endObject();
            }
            json.endArray();
            json.name("states").beginArray();
            for (BlockState state : block.getStateDefinition().getPossibleStates()) {
                int stateId = Block.getId(state);
                if (stateId != expectedState) {
                    throw new IllegalStateException("state id " + stateId + " expected " + expectedState);
                }
                expectedState++;
                writeState(json, state, stateId, shapes);
            }
            json.endArray();
            json.endObject();
        }
        json.endArray();
    }

    private static void writeState(JsonWriter json, BlockState state, int stateId, Map<List<Double>, Integer> shapes) throws IOException {
        FluidState fluid = state.getFluidState();
        VoxelShape collision = state.getCollisionShape(EmptyBlockGetter.INSTANCE, BlockPos.ZERO);
        json.beginObject();
        json.name("id").value(stateId);
        json.name("properties").beginObject();
        for (Property<?> property : state.getProperties()) {
            json.name(property.getName()).value(valueName(state, property));
        }
        json.endObject();
        json.name("light_emission").value(state.getLightEmission());
        json.name("light_dampening").value(state.getLightDampening());
        json.name("is_air").value(state.isAir());
        json.name("blocks_motion").value(state.is(BlockTags.BLOCKS_MOTION_IN_HEIGHTMAP));
        json.name("has_collision").value(!collision.isEmpty());
        json.name("can_occlude").value(state.canOcclude());
        json.name("is_solid_render").value(state.isSolidRender());
        json.name("propagates_skylight_down").value(state.propagatesSkylightDown());
        json.name("can_be_replaced").value(state.canBeReplaced());
        json.name("water").value(fluid.getType().isSame(Fluids.WATER));
        json.name("lava").value(fluid.getType().isSame(Fluids.LAVA));
        json.name("is_randomly_ticking").value(state.isRandomlyTicking());
        json.name("is_collision_shape_full_block").value(state.isCollisionShapeFullBlock(EmptyBlockGetter.INSTANCE, BlockPos.ZERO));
        json.name("is_solid").value(state.isSolid());
        json.name("is_redstone_conductor").value(state.isRedstoneConductor(EmptyBlockGetter.INSTANCE, BlockPos.ZERO));
        json.name("is_signal_source").value(state.isSignalSource());
        json.name("pathfindable_land").value(state.isPathfindable(PathComputationType.LAND));
        json.name("collision_shape").value(internShape(shapes, collision));
        json.name("sturdy_full").value(sturdyMask(state, SupportType.FULL));
        json.name("sturdy_center").value(sturdyMask(state, SupportType.CENTER));
        json.name("sturdy_rigid").value(sturdyMask(state, SupportType.RIGID));
        json.endObject();
    }

    private static String spawnSurface(Block block) {
        EntityType<?>[] probes = {EntityTypes.PIG, EntityTypes.OCELOT, EntityTypes.PARROT, EntityTypes.POLAR_BEAR, EntityTypes.STRIDER, EntityTypes.BLAZE};
        for (String surface : SPAWN_SURFACES) {
            boolean matches = true;
            for (BlockState state : block.getStateDefinition().getPossibleStates()) {
                for (EntityType<?> type : probes) {
                    if (state.isValidSpawn(EmptyBlockGetter.INSTANCE, BlockPos.ZERO, type) != expectedSpawn(surface, state, type)) {
                        matches = false;
                    }
                }
            }
            if (matches) {
                return surface;
            }
        }
        throw new IllegalStateException("unclassified isValidSpawn for " + BuiltInRegistries.BLOCK.getKey(block));
    }

    private static boolean expectedSpawn(String surface, BlockState state, EntityType<?> type) {
        return switch (surface) {
            case "default" -> state.isFaceSturdy(EmptyBlockGetter.INSTANCE, BlockPos.ZERO, Direction.UP) && state.getLightEmission() < 14;
            case "never" -> false;
            case "always" -> true;
            case "ocelot_or_parrot" -> type == EntityTypes.OCELOT || type == EntityTypes.PARROT;
            case "polar_bear" -> type == EntityTypes.POLAR_BEAR;
            case "fire_immune" -> type.fireImmune();
            default -> throw new IllegalStateException("unknown spawn surface " + surface);
        };
    }

    private static int internShape(Map<List<Double>, Integer> shapes, VoxelShape shape) {
        List<Double> key = new ArrayList<>();
        for (AABB box : shape.toAabbs()) {
            key.add(box.minX);
            key.add(box.minY);
            key.add(box.minZ);
            key.add(box.maxX);
            key.add(box.maxY);
            key.add(box.maxZ);
        }
        return shapes.computeIfAbsent(List.copyOf(key), ignored -> shapes.size());
    }

    private static int sturdyMask(BlockState state, SupportType support) {
        int mask = 0;
        for (int index = 0; index < FACES.length; index++) {
            if (state.isFaceSturdy(EmptyBlockGetter.INSTANCE, BlockPos.ZERO, FACES[index], support)) {
                mask |= 1 << index;
            }
        }
        return mask;
    }

    private static void writeShapes(JsonWriter json, Map<List<Double>, Integer> shapes) throws IOException {
        json.beginArray();
        int expected = 0;
        for (Map.Entry<List<Double>, Integer> entry : shapes.entrySet()) {
            if (entry.getValue() != expected) {
                throw new IllegalStateException("shape id " + entry.getValue() + " expected " + expected);
            }
            expected++;
            List<Double> values = entry.getKey();
            json.beginArray();
            for (int box = 0; box < values.size(); box += 6) {
                json.beginArray();
                for (int axis = 0; axis < 6; axis++) {
                    json.value(values.get(box + axis).doubleValue());
                }
                json.endArray();
            }
            json.endArray();
        }
        json.endArray();
    }

    private static Block wallBlock(StandingAndWallBlockItem item) {
        Map<Block, Item> registered = new LinkedHashMap<>();
        item.registerBlocks(registered, item);
        List<Block> others = new ArrayList<>();
        for (Block candidate : registered.keySet()) {
            if (candidate != item.getBlock()) {
                others.add(candidate);
            }
        }
        if (others.size() != 1) {
            throw new IllegalStateException("wall block of " + BuiltInRegistries.ITEM.getKey(item) + ": " + others);
        }
        return others.get(0);
    }

    private static <T extends Comparable<T>> List<String> valueNames(Property<T> property) {
        List<String> names = new ArrayList<>();
        for (T value : property.getPossibleValues()) {
            names.add(property.getName(value));
        }
        return names;
    }

    private static <T extends Comparable<T>> String valueName(BlockState state, Property<T> property) {
        return property.getName(state.getValue(property));
    }

    private static void writeBlockItems(JsonWriter json) throws IOException {
        json.beginArray();
        for (int id = 0; id < BuiltInRegistries.ITEM.size(); id++) {
            Item item = BuiltInRegistries.ITEM.byId(id);
            if (item instanceof BlockItem blockItem) {
                json.beginObject();
                json.name("name").value(BuiltInRegistries.ITEM.getKey(item).toString());
                json.name("id").value(id);
                json.name("block").value(BuiltInRegistries.BLOCK.getId(blockItem.getBlock()));
                json.endObject();
            }
        }
        json.endArray();
    }
}
