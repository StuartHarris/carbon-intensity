package com.example.counter.shared_types;


public final class GenerationMixPoint {
    public final String date;
    public final String hh_mm;
    public final String fuel;
    public final Float perc;

    public GenerationMixPoint(String date, String hh_mm, String fuel, Float perc) {
        java.util.Objects.requireNonNull(date, "date must not be null");
        java.util.Objects.requireNonNull(hh_mm, "hh_mm must not be null");
        java.util.Objects.requireNonNull(fuel, "fuel must not be null");
        java.util.Objects.requireNonNull(perc, "perc must not be null");
        this.date = date;
        this.hh_mm = hh_mm;
        this.fuel = fuel;
        this.perc = perc;
    }

    public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.increase_container_depth();
        serializer.serialize_str(date);
        serializer.serialize_str(hh_mm);
        serializer.serialize_str(fuel);
        serializer.serialize_f32(perc);
        serializer.decrease_container_depth();
    }

    public byte[] bincodeSerialize() throws com.novi.serde.SerializationError {
        com.novi.serde.Serializer serializer = new com.novi.bincode.BincodeSerializer();
        serialize(serializer);
        return serializer.get_bytes();
    }

    public static GenerationMixPoint deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        deserializer.increase_container_depth();
        Builder builder = new Builder();
        builder.date = deserializer.deserialize_str();
        builder.hh_mm = deserializer.deserialize_str();
        builder.fuel = deserializer.deserialize_str();
        builder.perc = deserializer.deserialize_f32();
        deserializer.decrease_container_depth();
        return builder.build();
    }

    public static GenerationMixPoint bincodeDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
        if (input == null) {
             throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
        }
        com.novi.serde.Deserializer deserializer = new com.novi.bincode.BincodeDeserializer(input);
        GenerationMixPoint value = deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.length) {
             throw new com.novi.serde.DeserializationError("Some input bytes were not read");
        }
        return value;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        GenerationMixPoint other = (GenerationMixPoint) obj;
        if (!java.util.Objects.equals(this.date, other.date)) { return false; }
        if (!java.util.Objects.equals(this.hh_mm, other.hh_mm)) { return false; }
        if (!java.util.Objects.equals(this.fuel, other.fuel)) { return false; }
        if (!java.util.Objects.equals(this.perc, other.perc)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + (this.date != null ? this.date.hashCode() : 0);
        value = 31 * value + (this.hh_mm != null ? this.hh_mm.hashCode() : 0);
        value = 31 * value + (this.fuel != null ? this.fuel.hashCode() : 0);
        value = 31 * value + (this.perc != null ? this.perc.hashCode() : 0);
        return value;
    }

    public static final class Builder {
        public String date;
        public String hh_mm;
        public String fuel;
        public Float perc;

        public GenerationMixPoint build() {
            return new GenerationMixPoint(
                date,
                hh_mm,
                fuel,
                perc
            );
        }
    }
}
