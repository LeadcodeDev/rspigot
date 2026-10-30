import com.google.gson.stream.JsonWriter;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import net.minecraft.SharedConstants;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.effect.MobEffect;

public final class EffectDump {
    private static final int MIN_DURATION = -2;
    private static final int MAX_DURATION = 400;
    private static final int[] AMPLIFIERS = {0, 1, 2, 3, 4, 5, 6, 7, 31, 32, 33, 255};

    private record Modifier(String attribute, String id, double amount, String operation) {
    }

    private EffectDump() {
    }

    public static void main(String[] args) throws IOException {
        if (args.length != 1) {
            System.err.println("usage: EffectDump <output.json>");
            System.exit(2);
        }
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        Path out = Path.of(args[0]).toAbsolutePath();
        Files.createDirectories(out.getParent());
        Path tmp = out.resolveSibling(out.getFileName() + ".tmp");
        try (JsonWriter json = new JsonWriter(Files.newBufferedWriter(tmp, StandardCharsets.UTF_8))) {
            json.setIndent(" ");
            json.beginObject();
            json.name("version").value(SharedConstants.getCurrentVersion().name());
            json.name("tick_durations").beginArray().value(MIN_DURATION).value(MAX_DURATION).endArray();
            json.name("tick_amplifiers").beginArray();
            for (int amplifier : AMPLIFIERS) {
                json.value(amplifier);
            }
            json.endArray();
            json.name("effects");
            writeEffects(json);
            json.endObject();
        }
        Files.move(tmp, out, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
        System.err.println("wrote " + out);
    }

    private static void writeEffects(JsonWriter json) throws IOException {
        json.beginArray();
        for (int id = 0; id < BuiltInRegistries.MOB_EFFECT.size(); id++) {
            MobEffect effect = BuiltInRegistries.MOB_EFFECT.byId(id);
            if (effect == null || BuiltInRegistries.MOB_EFFECT.getId(effect) != id) {
                throw new IllegalStateException("registry id mismatch at " + id);
            }
            json.beginObject();
            json.name("name").value(BuiltInRegistries.MOB_EFFECT.getKey(effect).toString());
            json.name("id").value(id);
            json.name("class").value(effect.getClass().getSimpleName());
            json.name("description_id").value(effect.getDescriptionId());
            json.name("category").value(effect.getCategory().name());
            json.name("beneficial").value(effect.isBeneficial());
            json.name("color").value(effect.getColor());
            json.name("instantaneous").value(effect.isInstantaneous());
            json.name("modifiers").beginArray();
            for (Modifier modifier : modifiers(effect)) {
                json.beginObject();
                json.name("attribute").value(modifier.attribute());
                json.name("id").value(modifier.id());
                json.name("amount").value(modifier.amount());
                json.name("operation").value(modifier.operation());
                json.endObject();
            }
            json.endArray();
            json.name("ticks").beginArray();
            for (int amplifier : AMPLIFIERS) {
                StringBuilder samples = new StringBuilder();
                for (int duration = MIN_DURATION; duration <= MAX_DURATION; duration++) {
                    samples.append(effect.shouldApplyEffectTickThisTick(duration, amplifier) ? '1' : '0');
                }
                json.value(samples.toString());
            }
            json.endArray();
            json.endObject();
        }
        json.endArray();
    }

    private static List<Modifier> modifiers(MobEffect effect) {
        List<Modifier> modifiers = new ArrayList<>();
        effect.createModifiers(0, (attribute, modifier) -> modifiers.add(new Modifier(
            BuiltInRegistries.ATTRIBUTE.getKey(attribute.value()).toString(),
            modifier.id().toString(),
            modifier.amount(),
            modifier.operation().getSerializedName())));
        modifiers.sort(Comparator.comparing(Modifier::attribute).thenComparing(Modifier::id));
        return modifiers;
    }
}
