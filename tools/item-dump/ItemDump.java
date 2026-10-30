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
import java.util.Optional;
import java.util.TreeMap;
import net.minecraft.SharedConstants;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.HolderSet;
import net.minecraft.core.Registry;
import net.minecraft.core.component.DataComponentInitializers;
import net.minecraft.core.component.DataComponentMap;
import net.minecraft.core.component.DataComponents;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.server.RegistryLayer;
import net.minecraft.server.packs.PackType;
import net.minecraft.server.packs.repository.ServerPacksSource;
import net.minecraft.server.packs.resources.MultiPackResourceManager;
import net.minecraft.tags.BlockTags;
import net.minecraft.tags.TagKey;
import net.minecraft.tags.TagLoader;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.food.FoodProperties;
import net.minecraft.world.item.BlockItem;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStackTemplate;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.Rarity;
import net.minecraft.world.item.StandingAndWallBlockItem;
import net.minecraft.world.item.component.DamageResistant;
import net.minecraft.world.item.component.ItemAttributeModifiers;
import net.minecraft.world.item.component.Tool;
import net.minecraft.world.item.component.UseRemainder;
import net.minecraft.world.item.enchantment.Enchantable;
import net.minecraft.world.item.enchantment.Repairable;
import net.minecraft.world.item.equipment.Equippable;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;

public final class ItemDump {
    private static final Map<String, TagKey<Block>> BLOCK_TAGS = new TreeMap<>();
    private static final Map<String, TagKey<Item>> ITEM_TAGS = new TreeMap<>();

    private ItemDump() {
    }

    public static void main(String[] args) throws IOException {
        if (args.length != 1) {
            System.err.println("usage: ItemDump <output.json>");
            System.exit(2);
        }
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        bindTags();
        bindComponents();
        Path out = Path.of(args[0]).toAbsolutePath();
        Files.createDirectories(out.getParent());
        Path tmp = out.resolveSibling(out.getFileName() + ".tmp");
        try (JsonWriter json = new JsonWriter(Files.newBufferedWriter(tmp, StandardCharsets.UTF_8))) {
            json.setIndent(" ");
            json.beginObject();
            json.name("version").value(SharedConstants.getCurrentVersion().name());
            json.name("block_count").value(BuiltInRegistries.BLOCK.size());
            json.name("default_max_stack_size").value(Item.DEFAULT_MAX_STACK_SIZE);
            json.name("absolute_max_stack_size").value(Item.ABSOLUTE_MAX_STACK_SIZE);
            json.name("rarities").beginArray();
            for (Rarity rarity : Rarity.values()) {
                json.value(rarity.getSerializedName());
            }
            json.endArray();
            json.name("equipment_slots").beginArray();
            for (EquipmentSlot slot : EquipmentSlot.values()) {
                json.value(slot.getSerializedName());
            }
            json.endArray();
            json.name("items");
            writeItems(json);
            json.name("block_tags");
            writeBlockTags(json);
            json.name("item_tags");
            writeItemTags(json);
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
        if (!Blocks.OBSIDIAN.defaultBlockState().is(BlockTags.MINEABLE_WITH_PICKAXE)) {
            throw new IllegalStateException("block tags not bound");
        }
    }

    private static void bindComponents() {
        HolderLookup.Provider lookup = VanillaRegistries.createWorldLookup();
        BuiltInRegistries.DATA_COMPONENT_INITIALIZERS.build(lookup).forEach(DataComponentInitializers.PendingComponents::apply);
        if (!BuiltInRegistries.ITEM.wrapAsHolder(BuiltInRegistries.ITEM.byId(1)).areComponentsBound()) {
            throw new IllegalStateException("item components not bound");
        }
        Tool pickaxe = Items.DIAMOND_PICKAXE.components().get(DataComponents.TOOL);
        if (pickaxe == null || !pickaxe.isCorrectForDrops(Blocks.OBSIDIAN.defaultBlockState())) {
            throw new IllegalStateException("tool rules do not see bound tags");
        }
    }

    private static void writeItems(JsonWriter json) throws IOException {
        json.beginArray();
        for (int id = 0; id < BuiltInRegistries.ITEM.size(); id++) {
            Item item = BuiltInRegistries.ITEM.byId(id);
            if (BuiltInRegistries.ITEM.getId(item) != id) {
                throw new IllegalStateException("registry id mismatch at " + id);
            }
            String name = BuiltInRegistries.ITEM.getKey(item).toString();
            DataComponentMap components = item.components();
            json.beginObject();
            json.name("name").value(name);
            json.name("id").value(id);
            json.name("class").value(item.getClass().getSimpleName());
            json.name("class_name").value(item.getClass().getName());
            json.name("block_item").value(item instanceof BlockItem);
            json.name("block");
            if (item instanceof BlockItem blockItem) {
                json.value(BuiltInRegistries.BLOCK.getId(blockItem.getBlock()));
            } else {
                json.nullValue();
            }
            json.name("wall_block");
            if (item instanceof StandingAndWallBlockItem standing) {
                json.value(BuiltInRegistries.BLOCK.getId(wallBlock(standing)));
            } else {
                json.nullValue();
            }
            json.name("default_max_stack_size").value(item.getDefaultMaxStackSize());
            json.name("max_stack_size").value(required(components.get(DataComponents.MAX_STACK_SIZE), name, "max_stack_size").intValue());
            json.name("max_damage");
            Integer maxDamage = components.get(DataComponents.MAX_DAMAGE);
            if (maxDamage == null) {
                json.nullValue();
            } else {
                json.value(maxDamage.intValue());
            }
            json.name("rarity").value(required(components.get(DataComponents.RARITY), name, "rarity").getSerializedName());
            json.name("tool");
            writeTool(json, components.get(DataComponents.TOOL));
            json.name("food");
            writeFood(json, components.get(DataComponents.FOOD));
            json.name("equippable");
            Equippable equippable = components.get(DataComponents.EQUIPPABLE);
            if (equippable == null) {
                json.nullValue();
            } else {
                json.value(equippable.slot().getSerializedName());
            }
            json.name("enchantable");
            Enchantable enchantable = components.get(DataComponents.ENCHANTABLE);
            if (enchantable == null) {
                json.nullValue();
            } else {
                json.value(enchantable.value());
            }
            json.name("repairable");
            Repairable repairable = components.get(DataComponents.REPAIRABLE);
            if (repairable == null) {
                json.nullValue();
            } else {
                writeItemSet(json, repairable.items());
            }
            json.name("damage_resistant");
            DamageResistant resistant = components.get(DataComponents.DAMAGE_RESISTANT);
            if (resistant == null) {
                json.nullValue();
            } else {
                Optional<? extends TagKey<?>> key = resistant.types().unwrapKey();
                if (key.isEmpty()) {
                    throw new IllegalStateException("direct damage_resistant set on " + name);
                }
                json.beginObject();
                json.name("tag").value(key.get().location().toString());
                json.endObject();
            }
            json.name("use_remainder");
            writeRemainder(json, components.get(DataComponents.USE_REMAINDER));
            json.name("attribute_modifiers");
            writeModifiers(json, components.getOrDefault(DataComponents.ATTRIBUTE_MODIFIERS, ItemAttributeModifiers.EMPTY));
            json.endObject();
        }
        json.endArray();
    }

    private static <T> T required(T value, String item, String component) {
        if (value == null) {
            throw new IllegalStateException(item + " has no " + component);
        }
        return value;
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

    private static void writeTool(JsonWriter json, Tool tool) throws IOException {
        if (tool == null) {
            json.nullValue();
            return;
        }
        json.beginObject();
        json.name("rules").beginArray();
        for (Tool.Rule rule : tool.rules()) {
            json.beginObject();
            json.name("blocks");
            writeBlockSet(json, rule.blocks());
            json.name("speed");
            if (rule.speed().isPresent()) {
                json.value(rule.speed().get());
            } else {
                json.nullValue();
            }
            json.name("correct_for_drops");
            if (rule.correctForDrops().isPresent()) {
                json.value(rule.correctForDrops().get().booleanValue());
            } else {
                json.nullValue();
            }
            json.endObject();
        }
        json.endArray();
        json.name("default_mining_speed").value(Float.valueOf(tool.defaultMiningSpeed()));
        json.name("damage_per_block").value(tool.damagePerBlock());
        json.name("can_destroy_blocks_in_creative").value(tool.canDestroyBlocksInCreative());
        json.name("correct_for_drops").beginArray();
        for (int id = 0; id < BuiltInRegistries.BLOCK.size(); id++) {
            BlockState state = BuiltInRegistries.BLOCK.byId(id).defaultBlockState();
            if (tool.isCorrectForDrops(state)) {
                json.value(id);
            }
        }
        json.endArray();
        json.name("mining_speeds").beginArray();
        for (int id = 0; id < BuiltInRegistries.BLOCK.size(); id++) {
            BlockState state = BuiltInRegistries.BLOCK.byId(id).defaultBlockState();
            float speed = tool.getMiningSpeed(state);
            if (Float.compare(speed, tool.defaultMiningSpeed()) != 0) {
                json.beginArray();
                json.value(id);
                json.value(Float.valueOf(speed));
                json.endArray();
            }
        }
        json.endArray();
        json.endObject();
    }

    private static void writeBlockSet(JsonWriter json, HolderSet<Block> set) throws IOException {
        json.beginObject();
        Optional<TagKey<Block>> key = set.unwrapKey();
        if (key.isPresent()) {
            String location = key.get().location().toString();
            BLOCK_TAGS.put(location, key.get());
            json.name("tag").value(location);
        } else {
            json.name("blocks").beginArray();
            for (Holder<Block> holder : set) {
                json.value(BuiltInRegistries.BLOCK.getId(holder.value()));
            }
            json.endArray();
        }
        json.endObject();
    }

    private static void writeItemSet(JsonWriter json, HolderSet<Item> set) throws IOException {
        json.beginObject();
        Optional<TagKey<Item>> key = set.unwrapKey();
        if (key.isPresent()) {
            String location = key.get().location().toString();
            ITEM_TAGS.put(location, key.get());
            json.name("tag").value(location);
        } else {
            json.name("items").beginArray();
            for (Holder<Item> holder : set) {
                json.value(BuiltInRegistries.ITEM.getId(holder.value()));
            }
            json.endArray();
        }
        json.endObject();
    }

    private static void writeFood(JsonWriter json, FoodProperties food) throws IOException {
        if (food == null) {
            json.nullValue();
            return;
        }
        json.beginObject();
        json.name("nutrition").value(food.nutrition());
        json.name("saturation").value(Float.valueOf(food.saturation()));
        json.name("can_always_eat").value(food.canAlwaysEat());
        json.endObject();
    }

    private static void writeRemainder(JsonWriter json, UseRemainder remainder) throws IOException {
        if (remainder == null) {
            json.nullValue();
            return;
        }
        ItemStackTemplate template = remainder.convertInto();
        json.beginObject();
        json.name("item").value(BuiltInRegistries.ITEM.getId(template.item().value()));
        json.name("count").value(template.count());
        json.name("patch_empty").value(template.components().isEmpty());
        json.endObject();
    }

    private static void writeModifiers(JsonWriter json, ItemAttributeModifiers modifiers) throws IOException {
        json.beginArray();
        for (ItemAttributeModifiers.Entry entry : modifiers.modifiers()) {
            json.beginObject();
            json.name("attribute").value(BuiltInRegistries.ATTRIBUTE.getKey(entry.attribute().value()).toString());
            json.name("id").value(entry.modifier().id().toString());
            json.name("amount").value(entry.modifier().amount());
            json.name("operation").value(entry.modifier().operation().getSerializedName());
            json.name("slot").value(entry.slot().getSerializedName());
            json.endObject();
        }
        json.endArray();
    }

    private static void writeBlockTags(JsonWriter json) throws IOException {
        json.beginArray();
        for (Map.Entry<String, TagKey<Block>> entry : BLOCK_TAGS.entrySet()) {
            HolderSet.Named<Block> set = BuiltInRegistries.BLOCK.get(entry.getValue()).orElseThrow(() -> new IllegalStateException("unbound block tag " + entry.getKey()));
            List<Integer> ids = new ArrayList<>();
            for (Holder<Block> holder : set) {
                ids.add(BuiltInRegistries.BLOCK.getId(holder.value()));
            }
            ids.sort(null);
            json.beginObject();
            json.name("name").value(entry.getKey());
            json.name("blocks").beginArray();
            for (int id : ids) {
                json.value(id);
            }
            json.endArray();
            json.endObject();
        }
        json.endArray();
    }

    private static void writeItemTags(JsonWriter json) throws IOException {
        json.beginArray();
        for (Map.Entry<String, TagKey<Item>> entry : ITEM_TAGS.entrySet()) {
            HolderSet.Named<Item> set = BuiltInRegistries.ITEM.get(entry.getValue()).orElseThrow(() -> new IllegalStateException("unbound item tag " + entry.getKey()));
            List<Integer> ids = new ArrayList<>();
            for (Holder<Item> holder : set) {
                ids.add(BuiltInRegistries.ITEM.getId(holder.value()));
            }
            ids.sort(null);
            json.beginObject();
            json.name("name").value(entry.getKey());
            json.name("items").beginArray();
            for (int id : ids) {
                json.value(id);
            }
            json.endArray();
            json.endObject();
        }
        json.endArray();
    }
}
