import com.google.gson.stream.JsonWriter;
import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayList;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import net.minecraft.SharedConstants;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.component.DataComponentInitializers;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.syncher.EntityDataAccessor;
import net.minecraft.network.syncher.EntityDataSerializers;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EntityDimensions;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.EntityTypes;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.MobCategory;
import net.minecraft.world.entity.ai.attributes.Attribute;
import net.minecraft.world.entity.ai.attributes.AttributeSupplier;
import net.minecraft.world.entity.ai.attributes.DefaultAttributes;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.SpawnEggItem;

public final class EntityDump {
    private EntityDump() {
    }

    public static void main(String[] args) throws IOException, IllegalAccessException {
        if (args.length != 1) {
            System.err.println("usage: EntityDump <output.json>");
            System.exit(2);
        }
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        bindComponents();
        Map<EntityType<?>, Class<?>> classes = entityClasses();
        Map<EntityType<?>, Integer> eggs = spawnEggs();
        Path out = Path.of(args[0]).toAbsolutePath();
        Files.createDirectories(out.getParent());
        Path tmp = out.resolveSibling(out.getFileName() + ".tmp");
        try (JsonWriter json = new JsonWriter(Files.newBufferedWriter(tmp, StandardCharsets.UTF_8))) {
            json.setIndent(" ");
            json.beginObject();
            json.name("version").value(SharedConstants.getCurrentVersion().name());
            json.name("no_update_interval").value(EntityType.NO_UPDATE_INTERVAL);
            json.name("mob_categories");
            writeCategories(json);
            json.name("attributes");
            writeAttributes(json);
            json.name("entity_types");
            writeEntityTypes(json, classes, eggs);
            json.endObject();
        }
        Files.move(tmp, out, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
        System.err.println("wrote " + out);
    }

    private static void bindComponents() {
        HolderLookup.Provider lookup = VanillaRegistries.createWorldLookup();
        BuiltInRegistries.DATA_COMPONENT_INITIALIZERS.build(lookup).forEach(DataComponentInitializers.PendingComponents::apply);
        if (!BuiltInRegistries.ITEM.wrapAsHolder(BuiltInRegistries.ITEM.byId(1)).areComponentsBound()) {
            throw new IllegalStateException("item components not bound");
        }
    }

    private static Map<EntityType<?>, Class<?>> entityClasses() throws IllegalAccessException {
        Map<EntityType<?>, Class<?>> classes = new IdentityHashMap<>();
        for (Field field : EntityTypes.class.getDeclaredFields()) {
            int modifiers = field.getModifiers();
            if (!Modifier.isStatic(modifiers) || field.getType() != EntityType.class) {
                continue;
            }
            Type generic = field.getGenericType();
            if (!(generic instanceof ParameterizedType parameterized)) {
                throw new IllegalStateException("raw entity type field " + field.getName());
            }
            Type argument = parameterized.getActualTypeArguments()[0];
            if (!(argument instanceof Class<?> entityClass) || !Entity.class.isAssignableFrom(entityClass)) {
                throw new IllegalStateException("entity type field " + field.getName() + " has argument " + argument);
            }
            EntityType<?> type = (EntityType<?>) field.get(null);
            if (classes.put(type, entityClass) != null) {
                throw new IllegalStateException("entity type bound twice: " + field.getName());
            }
        }
        if (classes.size() != BuiltInRegistries.ENTITY_TYPE.size()) {
            throw new IllegalStateException(classes.size() + " entity type fields for " + BuiltInRegistries.ENTITY_TYPE.size() + " registered types");
        }
        return classes;
    }

    private static Map<EntityType<?>, Integer> spawnEggs() {
        Map<EntityType<?>, Integer> eggs = new IdentityHashMap<>();
        for (int id = 0; id < BuiltInRegistries.ITEM.size(); id++) {
            Item item = BuiltInRegistries.ITEM.byId(id);
            if (!(item instanceof SpawnEggItem)) {
                continue;
            }
            EntityType<?> type = SpawnEggItem.getType(new ItemStack(item));
            if (type == null) {
                throw new IllegalStateException("spawn egg without entity: " + BuiltInRegistries.ITEM.getKey(item));
            }
            if (eggs.put(type, id) != null) {
                throw new IllegalStateException("two spawn eggs for " + EntityType.getKey(type));
            }
            Optional<Holder<Item>> lookup = SpawnEggItem.byId(type);
            if (lookup.isEmpty() || lookup.get().value() != item) {
                throw new IllegalStateException("SpawnEggItem.byId disagrees for " + EntityType.getKey(type));
            }
        }
        return eggs;
    }

    private static void writeCategories(JsonWriter json) throws IOException {
        json.beginArray();
        for (MobCategory category : MobCategory.values()) {
            json.beginObject();
            json.name("name").value(category.getName());
            json.name("serialized_name").value(category.getSerializedName());
            json.name("ordinal").value(category.ordinal());
            json.name("max_instances_per_chunk").value(category.getMaxInstancesPerChunk());
            json.name("is_friendly").value(category.isFriendly());
            json.name("is_persistent").value(category.isPersistent());
            json.name("despawn_distance").value(category.getDespawnDistance());
            json.name("no_despawn_distance").value(category.getNoDespawnDistance());
            json.endObject();
        }
        json.endArray();
    }

    private static void writeAttributes(JsonWriter json) throws IOException {
        json.beginArray();
        for (int id = 0; id < BuiltInRegistries.ATTRIBUTE.size(); id++) {
            Attribute attribute = BuiltInRegistries.ATTRIBUTE.byId(id);
            json.beginObject();
            json.name("name").value(BuiltInRegistries.ATTRIBUTE.getKey(attribute).toString());
            json.name("id").value(id);
            json.name("default_value").value(attribute.getDefaultValue());
            json.name("syncable").value(attribute.isClientSyncable());
            json.endObject();
        }
        json.endArray();
    }

    private static void writeDataAccessors(JsonWriter json, Class<?> entityClass) throws IOException, IllegalAccessException {
        List<Class<?>> chain = new ArrayList<>();
        for (Class<?> ancestor = entityClass; ancestor != null && Entity.class.isAssignableFrom(ancestor); ancestor = ancestor.getSuperclass()) {
            chain.add(0, ancestor);
        }
        json.beginArray();
        for (Class<?> owner : chain) {
            for (Field field : owner.getDeclaredFields()) {
                if (!Modifier.isStatic(field.getModifiers()) || field.getType() != EntityDataAccessor.class) {
                    continue;
                }
                field.setAccessible(true);
                EntityDataAccessor<?> accessor = (EntityDataAccessor<?>) field.get(null);
                json.beginObject();
                json.name("id").value(accessor.id());
                json.name("serializer").value(EntityDataSerializers.getSerializedId(accessor.serializer()));
                json.name("owner").value(owner.getSimpleName());
                json.name("field").value(field.getName());
                json.endObject();
            }
        }
        json.endArray();
    }

    @SuppressWarnings("unchecked")
    private static void writeEntityTypes(JsonWriter json, Map<EntityType<?>, Class<?>> classes, Map<EntityType<?>, Integer> eggs) throws IOException, IllegalAccessException {
        json.beginArray();
        for (int id = 0; id < BuiltInRegistries.ENTITY_TYPE.size(); id++) {
            EntityType<?> type = BuiltInRegistries.ENTITY_TYPE.byId(id);
            if (BuiltInRegistries.ENTITY_TYPE.getId(type) != id) {
                throw new IllegalStateException("registry id mismatch at " + id);
            }
            Class<?> entityClass = classes.get(type);
            boolean living = LivingEntity.class.isAssignableFrom(entityClass);
            if (living != DefaultAttributes.hasSupplier(type)) {
                throw new IllegalStateException("living " + living + " but hasSupplier " + DefaultAttributes.hasSupplier(type) + " for " + EntityType.getKey(type));
            }
            EntityDimensions dimensions = type.getDimensions();
            if (dimensions.width() != type.getWidth() || dimensions.height() != type.getHeight()) {
                throw new IllegalStateException("dimensions disagree for " + EntityType.getKey(type));
            }
            json.beginObject();
            json.name("name").value(EntityType.getKey(type).toString());
            json.name("id").value(id);
            json.name("class").value(entityClass.getSimpleName());
            json.name("class_name").value(entityClass.getName());
            json.name("category").value(type.getCategory().getName());
            json.name("width").value(Float.valueOf(dimensions.width()));
            json.name("height").value(Float.valueOf(dimensions.height()));
            json.name("eye_height").value(Float.valueOf(dimensions.eyeHeight()));
            json.name("fixed").value(dimensions.fixed());
            json.name("fire_immune").value(type.fireImmune());
            json.name("can_summon").value(type.canSummon());
            json.name("can_serialize").value(type.canSerialize());
            json.name("can_spawn_far_from_player").value(type.canSpawnFarFromPlayer());
            json.name("client_tracking_range").value(type.clientTrackingRange());
            json.name("update_interval").value(type.updateInterval());
            json.name("has_update_interval").value(type.hasUpdateInterval());
            json.name("track_deltas").value(type.trackDeltas());
            json.name("allowed_in_peaceful").value(type.isAllowedInPeaceful());
            json.name("spawn_egg");
            Integer egg = eggs.get(type);
            if (egg == null) {
                json.nullValue();
            } else {
                json.value(egg.intValue());
            }
            json.name("living").value(living);
            json.name("ancestors").beginArray();
            for (Class<?> ancestor = entityClass; ancestor != null && Entity.class.isAssignableFrom(ancestor); ancestor = ancestor.getSuperclass()) {
                json.value(ancestor.getSimpleName());
            }
            json.endArray();
            json.name("data_accessors");
            writeDataAccessors(json, entityClass);
            json.name("default_attributes").beginArray();
            if (living) {
                AttributeSupplier supplier = DefaultAttributes.getSupplier((EntityType<? extends LivingEntity>) type);
                for (int attributeId = 0; attributeId < BuiltInRegistries.ATTRIBUTE.size(); attributeId++) {
                    Attribute attribute = BuiltInRegistries.ATTRIBUTE.byId(attributeId);
                    Holder<Attribute> holder = BuiltInRegistries.ATTRIBUTE.wrapAsHolder(attribute);
                    if (!supplier.hasAttribute(holder)) {
                        continue;
                    }
                    json.beginObject();
                    json.name("name").value(BuiltInRegistries.ATTRIBUTE.getKey(attribute).toString());
                    json.name("base").value(supplier.getBaseValue(holder));
                    json.name("value").value(supplier.getValue(holder));
                    json.endObject();
                }
            }
            json.endArray();
            json.endObject();
        }
        json.endArray();
    }
}
