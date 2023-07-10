package com.example.counter.shared_types;


public final class Coordinate {
    public final Double latitude;
    public final Double longitude;

    public Coordinate(Double latitude, Double longitude) {
        java.util.Objects.requireNonNull(latitude, "latitude must not be null");
        java.util.Objects.requireNonNull(longitude, "longitude must not be null");
        this.latitude = latitude;
        this.longitude = longitude;
    }

    public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.increase_container_depth();
        serializer.serialize_f64(latitude);
        serializer.serialize_f64(longitude);
        serializer.decrease_container_depth();
    }

    public byte[] bincodeSerialize() throws com.novi.serde.SerializationError {
        com.novi.serde.Serializer serializer = new com.novi.bincode.BincodeSerializer();
        serialize(serializer);
        return serializer.get_bytes();
    }

    public static Coordinate deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        deserializer.increase_container_depth();
        Builder builder = new Builder();
        builder.latitude = deserializer.deserialize_f64();
        builder.longitude = deserializer.deserialize_f64();
        deserializer.decrease_container_depth();
        return builder.build();
    }

    public static Coordinate bincodeDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
        if (input == null) {
             throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
        }
        com.novi.serde.Deserializer deserializer = new com.novi.bincode.BincodeDeserializer(input);
        Coordinate value = deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.length) {
             throw new com.novi.serde.DeserializationError("Some input bytes were not read");
        }
        return value;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        Coordinate other = (Coordinate) obj;
        if (!java.util.Objects.equals(this.latitude, other.latitude)) { return false; }
        if (!java.util.Objects.equals(this.longitude, other.longitude)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + (this.latitude != null ? this.latitude.hashCode() : 0);
        value = 31 * value + (this.longitude != null ? this.longitude.hashCode() : 0);
        return value;
    }

    public static final class Builder {
        public Double latitude;
        public Double longitude;

        public Coordinate build() {
            return new Coordinate(
                latitude,
                longitude
            );
        }
    }
}
