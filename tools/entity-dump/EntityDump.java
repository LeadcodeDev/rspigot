import com.google.gson.stream.JsonWriter;
import java.io.IOException;
import java.io.InputStream;
import java.lang.classfile.ClassFile;
import java.lang.classfile.ClassModel;
import java.lang.classfile.CodeElement;
import java.lang.classfile.Instruction;
import java.lang.classfile.Label;
import java.lang.classfile.MethodModel;
import java.lang.classfile.Opcode;
import java.lang.classfile.instruction.BranchInstruction;
import java.lang.classfile.instruction.ConstantInstruction;
import java.lang.classfile.instruction.ConvertInstruction;
import java.lang.classfile.instruction.FieldInstruction;
import java.lang.classfile.instruction.InvokeDynamicInstruction;
import java.lang.classfile.instruction.InvokeInstruction;
import java.lang.classfile.instruction.LabelTarget;
import java.lang.classfile.instruction.LoadInstruction;
import java.lang.classfile.instruction.NewObjectInstruction;
import java.lang.classfile.instruction.OperatorInstruction;
import java.lang.classfile.instruction.ReturnInstruction;
import java.lang.classfile.instruction.StackInstruction;
import java.lang.classfile.instruction.TypeCheckInstruction;
import java.lang.constant.DirectMethodHandleDesc;
import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Deque;
import java.util.HashMap;
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
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.sounds.SoundEvent;
import net.minecraft.sounds.SoundSource;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EntityDimensions;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.EntityTypes;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.Mob;
import net.minecraft.world.entity.MobCategory;
import net.minecraft.world.entity.PathfinderMob;
import net.minecraft.world.entity.SpawnPlacementType;
import net.minecraft.world.entity.SpawnPlacementTypes;
import net.minecraft.world.entity.SpawnPlacements;
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
        HolderLookup.Provider lookup = bindComponents();
        Sounds sounds = new Sounds(lookup);
        Spawns spawns = new Spawns(sounds);
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
            writeEntityTypes(json, classes, eggs, sounds, spawns);
            json.endObject();
        }
        Files.move(tmp, out, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
        System.err.println("wrote " + out);
    }

    private static HolderLookup.Provider bindComponents() {
        HolderLookup.Provider lookup = VanillaRegistries.createWorldLookup();
        BuiltInRegistries.DATA_COMPONENT_INITIALIZERS.build(lookup).forEach(DataComponentInitializers.PendingComponents::apply);
        if (!BuiltInRegistries.ITEM.wrapAsHolder(BuiltInRegistries.ITEM.byId(1)).areComponentsBound()) {
            throw new IllegalStateException("item components not bound");
        }
        return lookup;
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
    private static void writeEntityTypes(JsonWriter json, Map<EntityType<?>, Class<?>> classes, Map<EntityType<?>, Integer> eggs, Sounds sounds, Spawns spawns) throws IOException, IllegalAccessException {
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
            if (living) {
                json.name("sounds");
                sounds.write(json, entityClass);
            }
            json.name("mob_spawn");
            spawns.write(json, type, entityClass);
            json.endObject();
        }
        spawns.checkAllWritten();
        json.endArray();
    }

    static final class Spawns {
        private static final String ENTITY_TYPES = "net/minecraft/world/entity/EntityTypes";
        private static final String PLACEMENT_TYPES = "net/minecraft/world/entity/SpawnPlacementTypes";
        private static final String HEIGHTMAP_TYPES = "net/minecraft/world/level/levelgen/Heightmap$Types";
        private static final String SPAWN_PLACEMENTS = "net/minecraft/world/entity/SpawnPlacements";
        private static final String LEVEL_ACCESSOR = "Lnet/minecraft/world/level/LevelAccessor;";
        private static final String LEVEL_READER = "Lnet/minecraft/world/level/LevelReader;";
        private static final String SPAWN_REASON = "Lnet/minecraft/world/entity/EntitySpawnReason;";
        private static final String BLOCK_POS = "Lnet/minecraft/core/BlockPos;";

        private final Sounds sounds;
        private final Map<EntityType<?>, String[]> registered = new IdentityHashMap<>();
        private final Map<EntityType<?>, Boolean> written = new IdentityHashMap<>();

        Spawns(Sounds sounds) throws IllegalAccessException {
            this.sounds = sounds;
            parseRegistrations();
            checkAgainstRuntime();
        }

        private void parseRegistrations() throws IllegalAccessException {
            MethodModel clinit = sounds.method(SpawnPlacements.class, "<clinit>", "()V");
            if (clinit == null) {
                throw new IllegalStateException("SpawnPlacements has no static initializer");
            }
            EntityType<?> type = null;
            String placement = null;
            String heightmap = null;
            String predicate = null;
            for (CodeElement element : clinit.code().orElseThrow().elementList()) {
                if (element instanceof FieldInstruction field && field.opcode() == Opcode.GETSTATIC) {
                    String owner = field.owner().asInternalName();
                    String name = field.name().stringValue();
                    if (owner.equals(ENTITY_TYPES)) {
                        type = entityType(name);
                    } else if (owner.equals(PLACEMENT_TYPES)) {
                        placement = name;
                    } else if (owner.equals(HEIGHTMAP_TYPES)) {
                        heightmap = name;
                    }
                } else if (element instanceof InvokeDynamicInstruction indy) {
                    if (indy.bootstrapArgs().size() < 2 || !(indy.bootstrapArgs().get(1) instanceof DirectMethodHandleDesc target)) {
                        throw new IllegalStateException("unexpected spawn predicate bootstrap " + indy);
                    }
                    predicate = target.owner().displayName() + "." + target.methodName();
                } else if (element instanceof InvokeInstruction invoke && invoke.opcode() == Opcode.INVOKESTATIC
                        && invoke.owner().asInternalName().equals(SPAWN_PLACEMENTS) && invoke.name().stringValue().equals("register")) {
                    if (type == null || placement == null || heightmap == null || predicate == null) {
                        throw new IllegalStateException("incomplete spawn placement registration before " + invoke);
                    }
                    if (registered.put(type, new String[] {placement, heightmap, predicate}) != null) {
                        throw new IllegalStateException("spawn placement registered twice for " + EntityType.getKey(type));
                    }
                    type = null;
                    placement = null;
                    heightmap = null;
                    predicate = null;
                }
            }
        }

        private static EntityType<?> entityType(String field) throws IllegalAccessException {
            try {
                return (EntityType<?>) EntityTypes.class.getField(field).get(null);
            } catch (NoSuchFieldException e) {
                throw new IllegalStateException("no entity type field " + field, e);
            }
        }

        private void checkAgainstRuntime() throws IllegalAccessException {
            Map<?, ?> runtime;
            try {
                Field data = SpawnPlacements.class.getDeclaredField("DATA_BY_TYPE");
                data.setAccessible(true);
                runtime = (Map<?, ?>) data.get(null);
            } catch (NoSuchFieldException e) {
                throw new IllegalStateException("SpawnPlacements.DATA_BY_TYPE not found", e);
            }
            if (runtime.size() != registered.size()) {
                throw new IllegalStateException(runtime.size() + " runtime placements but " + registered.size() + " parsed");
            }
            for (Map.Entry<EntityType<?>, String[]> entry : registered.entrySet()) {
                EntityType<?> type = entry.getKey();
                if (!runtime.containsKey(type)) {
                    throw new IllegalStateException("parsed placement missing at runtime for " + EntityType.getKey(type));
                }
                String placement = entry.getValue()[0];
                String heightmap = entry.getValue()[1];
                if (SpawnPlacements.getPlacementType(type) != placementType(placement)) {
                    throw new IllegalStateException("placement mismatch for " + EntityType.getKey(type));
                }
                if (!SpawnPlacements.getHeightmapType(type).name().equals(heightmap)) {
                    throw new IllegalStateException("heightmap mismatch for " + EntityType.getKey(type));
                }
            }
        }

        private static SpawnPlacementType placementType(String field) throws IllegalAccessException {
            try {
                return (SpawnPlacementType) SpawnPlacementTypes.class.getField(field).get(null);
            } catch (NoSuchFieldException e) {
                throw new IllegalStateException("no spawn placement type " + field, e);
            }
        }

        void write(JsonWriter json, EntityType<?> type, Class<?> entityClass) throws IOException, IllegalAccessException {
            if (!Mob.class.isAssignableFrom(entityClass)) {
                if (registered.containsKey(type)) {
                    throw new IllegalStateException("spawn placement for non-mob " + EntityType.getKey(type));
                }
                json.nullValue();
                return;
            }
            written.put(type, Boolean.TRUE);
            json.beginObject();
            json.name("placement");
            String[] placement = registered.get(type);
            if (placement == null) {
                json.nullValue();
            } else {
                json.beginObject();
                json.name("type").value(placement[0].toLowerCase(java.util.Locale.ROOT));
                json.name("heightmap").value(placement[1]);
                json.name("predicate").value(placement[2]);
                json.endObject();
            }
            json.name("check_spawn_rules").value(owner(entityClass, "checkSpawnRules", "(" + LEVEL_ACCESSOR + SPAWN_REASON + ")Z"));
            json.name("walk_target_value");
            if (PathfinderMob.class.isAssignableFrom(entityClass)) {
                json.value(owner(entityClass, "getWalkTargetValue", "(" + BLOCK_POS + LEVEL_READER + ")F"));
            } else {
                json.nullValue();
            }
            json.name("check_spawn_obstruction").value(owner(entityClass, "checkSpawnObstruction", "(" + LEVEL_READER + ")Z"));
            Class<?> clusterOwner = sounds.declaring(entityClass, "getMaxSpawnClusterSize", "()I");
            json.name("max_spawn_cluster_size_owner").value(clusterOwner.getSimpleName());
            json.name("max_spawn_cluster_size");
            Integer cluster = constantInt(sounds.method(clusterOwner, "getMaxSpawnClusterSize", "()I"));
            if (cluster == null) {
                json.nullValue();
            } else {
                json.value(cluster.intValue());
            }
            json.name("is_max_group_size_reached").value(owner(entityClass, "isMaxGroupSizeReached", "(I)Z"));
            json.name("remove_when_far_away").value(owner(entityClass, "removeWhenFarAway", "(D)Z"));
            json.name("requires_custom_persistence").value(owner(entityClass, "requiresCustomPersistence", "()Z"));
            json.name("spawn_dimensions_scale").value(Float.valueOf(spawnDimensionsScale(type)));
            json.name("immune_to");
            Object immune = typeField(type, "immuneTo");
            if (immune == null) {
                json.nullValue();
            } else {
                json.value(((net.minecraft.tags.TagKey<?>) immune).location().toString());
            }
            json.endObject();
        }

        void checkAllWritten() {
            for (EntityType<?> type : registered.keySet()) {
                if (!written.containsKey(type)) {
                    throw new IllegalStateException("spawn placement not written for " + EntityType.getKey(type));
                }
            }
        }

        private String owner(Class<?> entityClass, String name, String descriptor) {
            return sounds.declaring(entityClass, name, descriptor).getSimpleName();
        }

        private static Integer constantInt(MethodModel method) {
            List<Instruction> instructions = new ArrayList<>();
            for (CodeElement element : method.code().orElseThrow().elementList()) {
                if (element instanceof Instruction instruction) {
                    instructions.add(instruction);
                }
            }
            if (instructions.size() == 2 && instructions.get(0) instanceof ConstantInstruction constant
                    && instructions.get(1) instanceof ReturnInstruction && constant.constantValue() instanceof Integer value) {
                return value;
            }
            return null;
        }

        private static float spawnDimensionsScale(EntityType<?> type) throws IllegalAccessException {
            return ((Float) typeField(type, "spawnDimensionsScale")).floatValue();
        }

        private static Object typeField(EntityType<?> type, String name) throws IllegalAccessException {
            try {
                Field field = EntityType.class.getDeclaredField(name);
                field.setAccessible(true);
                return field.get(type);
            } catch (NoSuchFieldException e) {
                throw new IllegalStateException("EntityType has no field " + name, e);
            }
        }
    }

    static final class Sounds {
        private static final String SOUND_EVENT = "Lnet/minecraft/sounds/SoundEvent;";
        private static final String DAMAGE_SOURCE = "Lnet/minecraft/world/damagesource/DamageSource;";
        private static final Object THIS = new Object();
        private static final Object SOURCE = new Object();
        private static final Object SOURCE_TYPE = new Object();
        private static final Map<String, String> SOUND_VARIANT_REGISTRIES = Map.of(
            "Cow", "cow_sound_variant",
            "Pig", "pig_sound_variant",
            "Chicken", "chicken_sound_variant",
            "Cat", "cat_sound_variant",
            "Wolf", "wolf_sound_variant");
        private static final Map<String, Boolean> UNMODELED_FLAGS = Map.of(
            "isScared", false,
            "isClosed", true,
            "isScreamingGoat", false,
            "isHeartBound", false,
            "canArmorAbsorb", false);
        private static final Map<String, String> UNMODELED_ENUMS = Map.of(
            "getWeatherState", "UNAFFECTED",
            "getVariant", "DEFAULT",
            "effects", "HURT");
        private static final Map<String, String> PITCH_RULES = Map.of(
            "LivingEntity", "standard",
            "Player", "fixed",
            "HappyGhast", "fixed",
            "Bat", "bat",
            "ZombieVillager", "zombie_villager",
            "Parrot", "parrot");
        private static final String[] WATER_STATES = {"dry", "water", "submerged"};

        private final HolderLookup.Provider lookup;
        private final Map<Class<?>, ClassModel> models = new HashMap<>();

        Sounds(HolderLookup.Provider lookup) {
            this.lookup = lookup;
        }

        void write(JsonWriter json, Class<?> entityClass) throws IOException {
            json.beginObject();
            json.name("hurt");
            writeSounds(json, entityClass, "getHurtSound", "(" + DAMAGE_SOURCE + ")" + SOUND_EVENT);
            json.name("death");
            writeSounds(json, entityClass, "getDeathSound", "()" + SOUND_EVENT);
            Object fall = evaluate(entityClass, "getFallSounds", "()Lnet/minecraft/world/entity/LivingEntity$Fallsounds;", state(false, 2, "dry"));
            json.name("small_fall").value(soundName((SoundEvent) reflect(fall, "net/minecraft/world/entity/LivingEntity$Fallsounds", "small", new Object[0])));
            json.name("big_fall").value(soundName((SoundEvent) reflect(fall, "net/minecraft/world/entity/LivingEntity$Fallsounds", "big", new Object[0])));
            json.name("volume").beginObject();
            for (boolean baby : new boolean[] {false, true}) {
                for (int size = 1; size <= 2; size++) {
                    Object value = evaluate(entityClass, "getSoundVolume", "()F", state(baby, size, "dry"));
                    json.name((baby ? "baby" : "adult") + "_" + size).value(((Float) value).doubleValue());
                }
            }
            json.endObject();
            Object source = evaluate(entityClass, "getSoundSource", "()Lnet/minecraft/sounds/SoundSource;", state(false, 2, "dry"));
            json.name("source").value(((SoundSource) source).getName());
            Class<?> pitchOwner = declaring(entityClass, "getVoicePitch", "()F");
            String pitch = PITCH_RULES.get(pitchOwner.getSimpleName());
            if (pitch == null) {
                throw new IllegalStateException("no pitch rule for " + pitchOwner.getName());
            }
            if (pitch.equals("fixed")) {
                Object value = evaluate(entityClass, "getVoicePitch", "()F", state(false, 2, "dry"));
                if (!Float.valueOf(1.0F).equals(value)) {
                    throw new IllegalStateException("fixed pitch " + value + " for " + entityClass.getName());
                }
            }
            json.name("pitch").value(pitch);
            json.name("can_be_baby").value(canBeBaby(entityClass));
            json.endObject();
        }

        private boolean canBeBaby(Class<?> entityClass) {
            if (!net.minecraft.world.entity.AgeableMob.class.isAssignableFrom(entityClass)) {
                return declaring(entityClass, "isBaby", "()Z") != LivingEntity.class;
            }
            Object value = evaluate(entityClass, "canBeABaby", "()Z", state(false, 2, "dry"));
            return ((Integer) value) != 0;
        }

        private void writeSounds(JsonWriter json, Class<?> entityClass, String name, String descriptor) throws IOException {
            json.beginObject();
            for (boolean baby : new boolean[] {false, true}) {
                for (boolean tiny : new boolean[] {false, true}) {
                    for (String water : WATER_STATES) {
                        Object value = evaluate(entityClass, name, descriptor, state(baby, tiny ? 1 : 2, water));
                        String key = (baby ? "baby" : "adult") + "_" + (tiny ? "tiny" : "large") + "_" + water;
                        if (value == null) {
                            json.name(key).nullValue();
                        } else {
                            json.name(key).value(soundName((SoundEvent) value));
                        }
                    }
                }
            }
            json.endObject();
        }

        private static String soundName(SoundEvent event) {
            Identifier key = BuiltInRegistries.SOUND_EVENT.getKey(event);
            if (key == null) {
                throw new IllegalStateException("unregistered sound " + event.location());
            }
            return key.toString();
        }

        private static Map<String, Object> state(boolean baby, int size, String water) {
            Map<String, Object> state = new HashMap<>();
            state.put("isBaby", baby ? 1 : 0);
            state.put("isTiny", size <= 1 ? 1 : 0);
            state.put("getSize", size);
            state.put("isInWater", water.equals("dry") ? 0 : 1);
            state.put("isUnderWater", water.equals("submerged") ? 1 : 0);
            return state;
        }

        private ClassModel model(Class<?> owner) {
            return models.computeIfAbsent(owner, c -> {
                String resource = c.getName().replace('.', '/') + ".class";
                try (InputStream in = c.getClassLoader().getResourceAsStream(resource)) {
                    if (in == null) {
                        throw new IllegalStateException("no class bytes for " + c.getName());
                    }
                    return ClassFile.of().parse(in.readAllBytes());
                } catch (IOException e) {
                    throw new IllegalStateException(e);
                }
            });
        }

        private MethodModel method(Class<?> owner, String name, String descriptor) {
            for (MethodModel method : model(owner).methods()) {
                if (method.methodName().equalsString(name) && method.methodType().equalsString(descriptor)) {
                    return method;
                }
            }
            return null;
        }

        private Class<?> declaring(Class<?> start, String name, String descriptor) {
            for (Class<?> c = start; c != null && c != Object.class; c = c.getSuperclass()) {
                MethodModel method = method(c, name, descriptor);
                if (method != null && method.code().isPresent()) {
                    return c;
                }
            }
            throw new IllegalStateException("no implementation of " + name + descriptor + " for " + start.getName());
        }

        private Object evaluate(Class<?> concrete, String name, String descriptor, Map<String, Object> state) {
            return invokeThis(concrete, concrete, name, descriptor, state);
        }

        private Object invokeThis(Class<?> concrete, Class<?> start, String name, String descriptor, Map<String, Object> state) {
            if (state.containsKey(name) && descriptor.startsWith("()")) {
                return state.get(name);
            }
            if (UNMODELED_FLAGS.containsKey(name) && descriptor.endsWith(")Z")) {
                return UNMODELED_FLAGS.get(name) ? 1 : 0;
            }
            if (UNMODELED_ENUMS.containsKey(name)) {
                return enumConstant(descriptor, UNMODELED_ENUMS.get(name));
            }
            if (name.equals("getSoundVariant") && descriptor.equals("()Lnet/minecraft/core/Holder;")) {
                return defaultSoundVariant(concrete);
            }
            Class<?> owner = declaring(start, name, descriptor);
            return run(concrete, owner, method(owner, name, descriptor), state);
        }

        private Object defaultSoundVariant(Class<?> concrete) {
            for (Class<?> c = concrete; c != null; c = c.getSuperclass()) {
                String registry = SOUND_VARIANT_REGISTRIES.get(c.getSimpleName());
                if (registry != null) {
                    ResourceKey<net.minecraft.core.Registry<Object>> key = ResourceKey.createRegistryKey(Identifier.withDefaultNamespace(registry));
                    return lookup.lookupOrThrow(key).getOrThrow(ResourceKey.create(key, Identifier.withDefaultNamespace("classic")));
                }
            }
            throw new IllegalStateException("no sound variant registry for " + concrete.getName());
        }

        private static Object enumConstant(String descriptor, String constant) {
            String type = descriptor.substring(descriptor.indexOf(')') + 2, descriptor.length() - 1).replace('/', '.');
            try {
                Class<?> owner = Class.forName(type, true, Sounds.class.getClassLoader());
                return owner.getField(constant).get(null);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException("no constant " + constant + " in " + type, e);
            }
        }

        private Object run(Class<?> concrete, Class<?> owner, MethodModel method, Map<String, Object> state) {
            List<CodeElement> code = method.code().orElseThrow().elementList();
            Map<Label, Integer> labels = new HashMap<>();
            for (int i = 0; i < code.size(); i++) {
                if (code.get(i) instanceof LabelTarget target) {
                    labels.put(target.label(), i);
                }
            }
            Deque<Object> stack = new ArrayDeque<>();
            int pc = 0;
            while (pc < code.size()) {
                CodeElement element = code.get(pc++);
                if (!(element instanceof Instruction instruction)) {
                    continue;
                }
                switch (instruction) {
                    case LoadInstruction load -> stack.push(load.slot() == 0 ? THIS : SOURCE);
                    case NewObjectInstruction created -> stack.push(new Pending(created.className().asInternalName()));
                    case StackInstruction duplicate when duplicate.opcode() == Opcode.DUP -> stack.push(stack.peek());
                    case ConstantInstruction constant -> stack.push(constant.constantValue());
                    case TypeCheckInstruction check -> {
                    }
                    case ConvertInstruction convert when convert.opcode() == Opcode.I2F -> stack.push((float) (Integer) stack.pop());
                    case OperatorInstruction operator when operator.opcode() == Opcode.FMUL -> {
                        float right = (Float) stack.pop();
                        float left = (Float) stack.pop();
                        stack.push(left * right);
                    }
                    case FieldInstruction field when field.opcode() == Opcode.GETSTATIC -> stack.push(staticField(field));
                    case InvokeInstruction invoke -> stack.push(invoke(concrete, invoke, stack, state));
                    case BranchInstruction branch -> {
                        if (jumps(branch, stack)) {
                            pc = labels.get(branch.target());
                        }
                    }
                    case ReturnInstruction ret -> {
                        return stack.pop();
                    }
                    default -> throw new IllegalStateException("unsupported " + instruction + " in " + owner.getName() + "." + method.methodName());
                }
            }
            throw new IllegalStateException("fell off " + owner.getName() + "." + method.methodName());
        }

        private static boolean jumps(BranchInstruction branch, Deque<Object> stack) {
            Opcode opcode = branch.opcode();
            switch (opcode) {
                case GOTO, GOTO_W:
                    return true;
                case IFEQ:
                    return ((Integer) stack.pop()) == 0;
                case IFNE:
                    return ((Integer) stack.pop()) != 0;
                case IF_ACMPEQ:
                case IF_ACMPNE: {
                    Object right = stack.pop();
                    Object left = stack.pop();
                    return (left == right) == (opcode == Opcode.IF_ACMPEQ);
                }
                default:
                    throw new IllegalStateException("unsupported branch " + opcode);
            }
        }

        private static Object staticField(FieldInstruction field) {
            String owner = field.owner().asInternalName().replace('/', '.');
            try {
                Field target = Class.forName(owner, true, Sounds.class.getClassLoader()).getDeclaredField(field.name().stringValue());
                target.setAccessible(true);
                return target.get(null);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException("cannot read " + owner + "." + field.name(), e);
            }
        }

        private Object invoke(Class<?> concrete, InvokeInstruction invoke, Deque<Object> stack, Map<String, Object> state) {
            String name = invoke.name().stringValue();
            String descriptor = invoke.type().stringValue();
            int count = invoke.typeSymbol().parameterCount();
            Object[] args = new Object[count];
            for (int i = count - 1; i >= 0; i--) {
                args[i] = stack.pop();
            }
            if (invoke.opcode() == Opcode.INVOKESTATIC) {
                return reflect(null, invoke.owner().asInternalName(), name, args);
            }
            Object receiver = stack.pop();
            if (receiver instanceof Pending pending && name.equals("<init>")) {
                Object built = construct(pending.owner(), args);
                stack.pop();
                return built;
            }
            if (receiver == THIS) {
                if (invoke.opcode() == Opcode.INVOKESPECIAL) {
                    try {
                        Class<?> owner = Class.forName(invoke.owner().asInternalName().replace('/', '.'), true, Sounds.class.getClassLoader());
                        return invokeThis(concrete, owner, name, descriptor, state);
                    } catch (ClassNotFoundException e) {
                        throw new IllegalStateException(e);
                    }
                }
                return invokeThis(concrete, concrete, name, descriptor, state);
            }
            if (receiver == SOURCE) {
                if (name.equals("type")) {
                    return SOURCE_TYPE;
                }
                throw new IllegalStateException("unsupported damage source call " + name);
            }
            if (receiver == SOURCE_TYPE) {
                if (UNMODELED_ENUMS.containsKey(name)) {
                    return enumConstant(descriptor, UNMODELED_ENUMS.get(name));
                }
                throw new IllegalStateException("unsupported damage type call " + name);
            }
            return reflect(receiver, invoke.owner().asInternalName(), name, args);
        }

        private record Pending(String owner) {
        }

        private static Object construct(String owner, Object[] args) {
            try {
                Class<?> type = Class.forName(owner.replace('/', '.'), true, Sounds.class.getClassLoader());
                for (Constructor<?> constructor : type.getDeclaredConstructors()) {
                    if (constructor.getParameterCount() == args.length) {
                        constructor.setAccessible(true);
                        return constructor.newInstance(args);
                    }
                }
                throw new IllegalStateException("no constructor for " + owner);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException("cannot build " + owner, e);
            }
        }

        private static Object reflect(Object receiver, String owner, String name, Object[] args) {
            try {
                Class<?> type = receiver != null ? receiver.getClass() : Class.forName(owner.replace('/', '.'), true, Sounds.class.getClassLoader());
                for (Class<?> c = type; c != null; c = c.getSuperclass()) {
                    for (Method method : c.getDeclaredMethods()) {
                        if (method.getName().equals(name) && method.getParameterCount() == args.length) {
                            method.setAccessible(true);
                            return method.invoke(receiver, args);
                        }
                    }
                }
                Class<?> declared = Class.forName(owner.replace('/', '.'), true, Sounds.class.getClassLoader());
                for (Method method : declared.getMethods()) {
                    if (method.getName().equals(name) && method.getParameterCount() == args.length) {
                        return method.invoke(receiver, args);
                    }
                }
                throw new IllegalStateException("no method " + owner + "." + name);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException("cannot call " + owner + "." + name, e);
            }
        }
    }
}
