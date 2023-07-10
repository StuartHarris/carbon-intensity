package com.stuartharris.carbon.shared_types;

final class TraitHelpers {
    static void serialize_option_Coordinate(java.util.Optional<Coordinate> value, com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        if (value.isPresent()) {
            serializer.serialize_option_tag(true);
            value.get().serialize(serializer);
        } else {
            serializer.serialize_option_tag(false);
        }
    }

    static java.util.Optional<Coordinate> deserialize_option_Coordinate(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        boolean tag = deserializer.deserialize_option_tag();
        if (!tag) {
            return java.util.Optional.empty();
        } else {
            return java.util.Optional.of(Coordinate.deserialize(deserializer));
        }
    }

    static void serialize_option_i32(java.util.Optional<Integer> value, com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        if (value.isPresent()) {
            serializer.serialize_option_tag(true);
            serializer.serialize_i32(value.get());
        } else {
            serializer.serialize_option_tag(false);
        }
    }

    static java.util.Optional<Integer> deserialize_option_i32(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        boolean tag = deserializer.deserialize_option_tag();
        if (!tag) {
            return java.util.Optional.empty();
        } else {
            return java.util.Optional.of(deserializer.deserialize_i32());
        }
    }

    static void serialize_vector_GenerationMixPoint(java.util.List<GenerationMixPoint> value, com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.serialize_len(value.size());
        for (GenerationMixPoint item : value) {
            item.serialize(serializer);
        }
    }

    static java.util.List<GenerationMixPoint> deserialize_vector_GenerationMixPoint(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        long length = deserializer.deserialize_len();
        java.util.List<GenerationMixPoint> obj = new java.util.ArrayList<GenerationMixPoint>((int) length);
        for (long i = 0; i < length; i++) {
            obj.add(GenerationMixPoint.deserialize(deserializer));
        }
        return obj;
    }

    static void serialize_vector_HttpHeader(java.util.List<HttpHeader> value, com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.serialize_len(value.size());
        for (HttpHeader item : value) {
            item.serialize(serializer);
        }
    }

    static java.util.List<HttpHeader> deserialize_vector_HttpHeader(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        long length = deserializer.deserialize_len();
        java.util.List<HttpHeader> obj = new java.util.ArrayList<HttpHeader>((int) length);
        for (long i = 0; i < length; i++) {
            obj.add(HttpHeader.deserialize(deserializer));
        }
        return obj;
    }

    static void serialize_vector_IntensityPoint(java.util.List<IntensityPoint> value, com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.serialize_len(value.size());
        for (IntensityPoint item : value) {
            item.serialize(serializer);
        }
    }

    static java.util.List<IntensityPoint> deserialize_vector_IntensityPoint(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        long length = deserializer.deserialize_len();
        java.util.List<IntensityPoint> obj = new java.util.ArrayList<IntensityPoint>((int) length);
        for (long i = 0; i < length; i++) {
            obj.add(IntensityPoint.deserialize(deserializer));
        }
        return obj;
    }

    static void serialize_vector_u8(java.util.List<@com.novi.serde.Unsigned Byte> value, com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
        serializer.serialize_len(value.size());
        for (@com.novi.serde.Unsigned Byte item : value) {
            serializer.serialize_u8(item);
        }
    }

    static java.util.List<@com.novi.serde.Unsigned Byte> deserialize_vector_u8(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
        long length = deserializer.deserialize_len();
        java.util.List<@com.novi.serde.Unsigned Byte> obj = new java.util.ArrayList<@com.novi.serde.Unsigned Byte>((int) length);
        for (long i = 0; i < length; i++) {
            obj.add(deserializer.deserialize_u8());
        }
        return obj;
    }

}

