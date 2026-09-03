// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'restrictions.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$RestrictionDto {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RestrictionDto);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RestrictionDto()';
}


}

/// @nodoc
class $RestrictionDtoCopyWith<$Res>  {
$RestrictionDtoCopyWith(RestrictionDto _, $Res Function(RestrictionDto) __);
}


/// Adds pattern-matching-related methods to [RestrictionDto].
extension RestrictionDtoPatterns on RestrictionDto {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( RestrictionDto_Known value)?  known,TResult Function( RestrictionDto_Other value)?  other,required TResult orElse(),}){
final _that = this;
switch (_that) {
case RestrictionDto_Known() when known != null:
return known(_that);case RestrictionDto_Other() when other != null:
return other(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( RestrictionDto_Known value)  known,required TResult Function( RestrictionDto_Other value)  other,}){
final _that = this;
switch (_that) {
case RestrictionDto_Known():
return known(_that);case RestrictionDto_Other():
return other(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( RestrictionDto_Known value)?  known,TResult? Function( RestrictionDto_Other value)?  other,}){
final _that = this;
switch (_that) {
case RestrictionDto_Known() when known != null:
return known(_that);case RestrictionDto_Other() when other != null:
return other(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String kind)?  known,TResult Function( String text)?  other,required TResult orElse(),}) {final _that = this;
switch (_that) {
case RestrictionDto_Known() when known != null:
return known(_that.kind);case RestrictionDto_Other() when other != null:
return other(_that.text);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String kind)  known,required TResult Function( String text)  other,}) {final _that = this;
switch (_that) {
case RestrictionDto_Known():
return known(_that.kind);case RestrictionDto_Other():
return other(_that.text);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String kind)?  known,TResult? Function( String text)?  other,}) {final _that = this;
switch (_that) {
case RestrictionDto_Known() when known != null:
return known(_that.kind);case RestrictionDto_Other() when other != null:
return other(_that.text);case _:
  return null;

}
}

}

/// @nodoc


class RestrictionDto_Known extends RestrictionDto {
  const RestrictionDto_Known({required this.kind}): super._();
  

 final  String kind;

/// Create a copy of RestrictionDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RestrictionDto_KnownCopyWith<RestrictionDto_Known> get copyWith => _$RestrictionDto_KnownCopyWithImpl<RestrictionDto_Known>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RestrictionDto_Known&&(identical(other.kind, kind) || other.kind == kind));
}


@override
int get hashCode => Object.hash(runtimeType,kind);

@override
String toString() {
  return 'RestrictionDto.known(kind: $kind)';
}


}

/// @nodoc
abstract mixin class $RestrictionDto_KnownCopyWith<$Res> implements $RestrictionDtoCopyWith<$Res> {
  factory $RestrictionDto_KnownCopyWith(RestrictionDto_Known value, $Res Function(RestrictionDto_Known) _then) = _$RestrictionDto_KnownCopyWithImpl;
@useResult
$Res call({
 String kind
});




}
/// @nodoc
class _$RestrictionDto_KnownCopyWithImpl<$Res>
    implements $RestrictionDto_KnownCopyWith<$Res> {
  _$RestrictionDto_KnownCopyWithImpl(this._self, this._then);

  final RestrictionDto_Known _self;
  final $Res Function(RestrictionDto_Known) _then;

/// Create a copy of RestrictionDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? kind = null,}) {
  return _then(RestrictionDto_Known(
kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class RestrictionDto_Other extends RestrictionDto {
  const RestrictionDto_Other({required this.text}): super._();
  

 final  String text;

/// Create a copy of RestrictionDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RestrictionDto_OtherCopyWith<RestrictionDto_Other> get copyWith => _$RestrictionDto_OtherCopyWithImpl<RestrictionDto_Other>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RestrictionDto_Other&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'RestrictionDto.other(text: $text)';
}


}

/// @nodoc
abstract mixin class $RestrictionDto_OtherCopyWith<$Res> implements $RestrictionDtoCopyWith<$Res> {
  factory $RestrictionDto_OtherCopyWith(RestrictionDto_Other value, $Res Function(RestrictionDto_Other) _then) = _$RestrictionDto_OtherCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$RestrictionDto_OtherCopyWithImpl<$Res>
    implements $RestrictionDto_OtherCopyWith<$Res> {
  _$RestrictionDto_OtherCopyWithImpl(this._self, this._then);

  final RestrictionDto_Other _self;
  final $Res Function(RestrictionDto_Other) _then;

/// Create a copy of RestrictionDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(RestrictionDto_Other(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
