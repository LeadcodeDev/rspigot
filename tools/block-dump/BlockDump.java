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
import net.minecraft.core.Registry;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.server.RegistryLayer;
import net.minecraft.server.packs.PackType;
import net.minecraft.server.packs.repository.ServerPacksSource;
import net.minecraft.server.packs.resources.MultiPackResourceManager;
import net.minecraft.tags.BlockTags;
import net.minecraft.tags.TagLoader;
import net.minecraft.world.item.BlockItem;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.StandingAndWallBlockItem;
import net.minecraft.world.level.EmptyBlockGetter;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.Property;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.material.Fluids;

public final class BlockDump {
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
            json.name("blocks");
            writeBlocks(json);
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

    private static void writeBlocks(JsonWriter json) throws IOException {
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
            json.name("requires_correct_tool_for_drops").value(defaultState.requiresCorrectToolForDrops());
            json.name("ignited_by_lava").value(defaultState.ignitedByLava());
            json.name("default_state").value(Block.getId(defaultState));
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
                writeState(json, state, stateId);
            }
            json.endArray();
            json.endObject();
        }
        json.endArray();
    }

    private static void writeState(JsonWriter json, BlockState state, int stateId) throws IOException {
        FluidState fluid = state.getFluidState();
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
        json.name("has_collision").value(!state.getCollisionShape(EmptyBlockGetter.INSTANCE, BlockPos.ZERO).isEmpty());
        json.name("can_occlude").value(state.canOcclude());
        json.name("is_solid_render").value(state.isSolidRender());
        json.name("propagates_skylight_down").value(state.propagatesSkylightDown());
        json.name("can_be_replaced").value(state.canBeReplaced());
        json.name("water").value(fluid.getType().isSame(Fluids.WATER));
        json.name("lava").value(fluid.getType().isSame(Fluids.LAVA));
        json.name("is_randomly_ticking").value(state.isRandomlyTicking());
        json.name("is_collision_shape_full_block").value(state.isCollisionShapeFullBlock(EmptyBlockGetter.INSTANCE, BlockPos.ZERO));
        json.endObject();
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
