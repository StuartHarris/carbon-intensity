package com.stuartharris.carbon.shared_types;


public final class ViewModel {
    public final Mode mode;
    public final String national_name;
    public final java.util.List<IntensityPoint> national_intensity;
    public final java.util.List<GenerationMixPoint> national_mix;
    public final String local_name;
    public final java.util.List<IntensityPoint> local_intensity;
    public final java.util.List<GenerationMixPoint> local_mix;

    public ViewModel(Mode mode, String national_name, java.util.List<IntensityPoint> national_intensity, java.util.List<GenerationMixPoint> national_mix, String local_name, java.util.List<IntensityPoint> local_intensity, java.util.List<GenerationMixPoint> local_mix) {
        java.util.Objects.requireNonNull(mode, "mode must not be null");
        java.util.Objects.requireNonNull(national_name, "national_name must not be null");
        java.util.Objects.requireNonNull(national_intensity, "national_intensity must not be null");
        java.util.Objects.requireNonNull(national_mix, "national_mix must not be null");
        java.util.Objects.requireNonNull(local_name, "local_name must not be null");
        java.util.Objects.requireNonNull(local_intensity, "local_intensity must not be null");
        java.util.Objects.requireNonNull(local_mix, "local_mix must not be null");
        this.mode = mode;
        this.national_name = national_name;
        this.national_intensity = national_intensity;
        this.national_mix = national_mix;
        this.local_name = local_name;
        this.local_intensity = local_intensity;
        this.local_mix = local_mix;
    }

    public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.increase_container_depth();
        mode.serialize(serializer);
        serializer.serialize_str(national_name);
        TraitHelpers.serialize_vector_IntensityPoint(national_intensity, serializer);
        TraitHelpers.serialize_vector_GenerationMixPoint(national_mix, serializer);
        serializer.serialize_str(local_name);
        TraitHelpers.serialize_vector_IntensityPoint(local_intensity, serializer);
        TraitHelpers.serialize_vector_GenerationMixPoint(local_mix, serializer);
        serializer.decrease_container_depth();
    }

    public byte[] bincodeSerialize() throws com.novi.serde.SerializationError {
        com.novi.serde.Serializer serializer = new com.novi.bincode.BincodeSerializer();
        serialize(serializer);
        return serializer.get_bytes();
    }

    public static ViewModel deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        deserializer.increase_container_depth();
        Builder builder = new Builder();
        builder.mode = Mode.deserialize(deserializer);
        builder.national_name = deserializer.deserialize_str();
        builder.national_intensity = TraitHelpers.deserialize_vector_IntensityPoint(deserializer);
        builder.national_mix = TraitHelpers.deserialize_vector_GenerationMixPoint(deserializer);
        builder.local_name = deserializer.deserialize_str();
        builder.local_intensity = TraitHelpers.deserialize_vector_IntensityPoint(deserializer);
        builder.local_mix = TraitHelpers.deserialize_vector_GenerationMixPoint(deserializer);
        deserializer.decrease_container_depth();
        return builder.build();
    }

    public static ViewModel bincodeDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
        if (input == null) {
             throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
        }
        com.novi.serde.Deserializer deserializer = new com.novi.bincode.BincodeDeserializer(input);
        ViewModel value = deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.length) {
             throw new com.novi.serde.DeserializationError("Some input bytes were not read");
        }
        return value;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        ViewModel other = (ViewModel) obj;
        if (!java.util.Objects.equals(this.mode, other.mode)) { return false; }
        if (!java.util.Objects.equals(this.national_name, other.national_name)) { return false; }
        if (!java.util.Objects.equals(this.national_intensity, other.national_intensity)) { return false; }
        if (!java.util.Objects.equals(this.national_mix, other.national_mix)) { return false; }
        if (!java.util.Objects.equals(this.local_name, other.local_name)) { return false; }
        if (!java.util.Objects.equals(this.local_intensity, other.local_intensity)) { return false; }
        if (!java.util.Objects.equals(this.local_mix, other.local_mix)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + (this.mode != null ? this.mode.hashCode() : 0);
        value = 31 * value + (this.national_name != null ? this.national_name.hashCode() : 0);
        value = 31 * value + (this.national_intensity != null ? this.national_intensity.hashCode() : 0);
        value = 31 * value + (this.national_mix != null ? this.national_mix.hashCode() : 0);
        value = 31 * value + (this.local_name != null ? this.local_name.hashCode() : 0);
        value = 31 * value + (this.local_intensity != null ? this.local_intensity.hashCode() : 0);
        value = 31 * value + (this.local_mix != null ? this.local_mix.hashCode() : 0);
        return value;
    }

    public static final class Builder {
        public Mode mode;
        public String national_name;
        public java.util.List<IntensityPoint> national_intensity;
        public java.util.List<GenerationMixPoint> national_mix;
        public String local_name;
        public java.util.List<IntensityPoint> local_intensity;
        public java.util.List<GenerationMixPoint> local_mix;

        public ViewModel build() {
            return new ViewModel(
                mode,
                national_name,
                national_intensity,
                national_mix,
                local_name,
                local_intensity,
                local_mix
            );
        }
    }
}
