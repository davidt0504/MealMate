// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'error.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$KimattaError {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KimattaError);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KimattaError()';
}


}

/// @nodoc
class $KimattaErrorCopyWith<$Res>  {
$KimattaErrorCopyWith(KimattaError _, $Res Function(KimattaError) __);
}


/// Adds pattern-matching-related methods to [KimattaError].
extension KimattaErrorPatterns on KimattaError {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( KimattaError_InvalidPath value)?  invalidPath,TResult Function( KimattaError_NotOpen value)?  notOpen,TResult Function( KimattaError_Storage value)?  storage,TResult Function( KimattaError_Planning value)?  planning,TResult Function( KimattaError_Recipe value)?  recipe,TResult Function( KimattaError_Restriction value)?  restriction,required TResult orElse(),}){
final _that = this;
switch (_that) {
case KimattaError_InvalidPath() when invalidPath != null:
return invalidPath(_that);case KimattaError_NotOpen() when notOpen != null:
return notOpen(_that);case KimattaError_Storage() when storage != null:
return storage(_that);case KimattaError_Planning() when planning != null:
return planning(_that);case KimattaError_Recipe() when recipe != null:
return recipe(_that);case KimattaError_Restriction() when restriction != null:
return restriction(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( KimattaError_InvalidPath value)  invalidPath,required TResult Function( KimattaError_NotOpen value)  notOpen,required TResult Function( KimattaError_Storage value)  storage,required TResult Function( KimattaError_Planning value)  planning,required TResult Function( KimattaError_Recipe value)  recipe,required TResult Function( KimattaError_Restriction value)  restriction,}){
final _that = this;
switch (_that) {
case KimattaError_InvalidPath():
return invalidPath(_that);case KimattaError_NotOpen():
return notOpen(_that);case KimattaError_Storage():
return storage(_that);case KimattaError_Planning():
return planning(_that);case KimattaError_Recipe():
return recipe(_that);case KimattaError_Restriction():
return restriction(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( KimattaError_InvalidPath value)?  invalidPath,TResult? Function( KimattaError_NotOpen value)?  notOpen,TResult? Function( KimattaError_Storage value)?  storage,TResult? Function( KimattaError_Planning value)?  planning,TResult? Function( KimattaError_Recipe value)?  recipe,TResult? Function( KimattaError_Restriction value)?  restriction,}){
final _that = this;
switch (_that) {
case KimattaError_InvalidPath() when invalidPath != null:
return invalidPath(_that);case KimattaError_NotOpen() when notOpen != null:
return notOpen(_that);case KimattaError_Storage() when storage != null:
return storage(_that);case KimattaError_Planning() when planning != null:
return planning(_that);case KimattaError_Recipe() when recipe != null:
return recipe(_that);case KimattaError_Restriction() when restriction != null:
return restriction(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  invalidPath,TResult Function()?  notOpen,TResult Function( String message)?  storage,TResult Function( String message)?  planning,TResult Function( String message)?  recipe,TResult Function( String message)?  restriction,required TResult orElse(),}) {final _that = this;
switch (_that) {
case KimattaError_InvalidPath() when invalidPath != null:
return invalidPath();case KimattaError_NotOpen() when notOpen != null:
return notOpen();case KimattaError_Storage() when storage != null:
return storage(_that.message);case KimattaError_Planning() when planning != null:
return planning(_that.message);case KimattaError_Recipe() when recipe != null:
return recipe(_that.message);case KimattaError_Restriction() when restriction != null:
return restriction(_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  invalidPath,required TResult Function()  notOpen,required TResult Function( String message)  storage,required TResult Function( String message)  planning,required TResult Function( String message)  recipe,required TResult Function( String message)  restriction,}) {final _that = this;
switch (_that) {
case KimattaError_InvalidPath():
return invalidPath();case KimattaError_NotOpen():
return notOpen();case KimattaError_Storage():
return storage(_that.message);case KimattaError_Planning():
return planning(_that.message);case KimattaError_Recipe():
return recipe(_that.message);case KimattaError_Restriction():
return restriction(_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  invalidPath,TResult? Function()?  notOpen,TResult? Function( String message)?  storage,TResult? Function( String message)?  planning,TResult? Function( String message)?  recipe,TResult? Function( String message)?  restriction,}) {final _that = this;
switch (_that) {
case KimattaError_InvalidPath() when invalidPath != null:
return invalidPath();case KimattaError_NotOpen() when notOpen != null:
return notOpen();case KimattaError_Storage() when storage != null:
return storage(_that.message);case KimattaError_Planning() when planning != null:
return planning(_that.message);case KimattaError_Recipe() when recipe != null:
return recipe(_that.message);case KimattaError_Restriction() when restriction != null:
return restriction(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class KimattaError_InvalidPath extends KimattaError {
  const KimattaError_InvalidPath(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KimattaError_InvalidPath);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KimattaError.invalidPath()';
}


}




/// @nodoc


class KimattaError_NotOpen extends KimattaError {
  const KimattaError_NotOpen(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KimattaError_NotOpen);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KimattaError.notOpen()';
}


}




/// @nodoc


class KimattaError_Storage extends KimattaError {
  const KimattaError_Storage({required this.message}): super._();
  

 final  String message;

/// Create a copy of KimattaError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KimattaError_StorageCopyWith<KimattaError_Storage> get copyWith => _$KimattaError_StorageCopyWithImpl<KimattaError_Storage>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KimattaError_Storage&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KimattaError.storage(message: $message)';
}


}

/// @nodoc
abstract mixin class $KimattaError_StorageCopyWith<$Res> implements $KimattaErrorCopyWith<$Res> {
  factory $KimattaError_StorageCopyWith(KimattaError_Storage value, $Res Function(KimattaError_Storage) _then) = _$KimattaError_StorageCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KimattaError_StorageCopyWithImpl<$Res>
    implements $KimattaError_StorageCopyWith<$Res> {
  _$KimattaError_StorageCopyWithImpl(this._self, this._then);

  final KimattaError_Storage _self;
  final $Res Function(KimattaError_Storage) _then;

/// Create a copy of KimattaError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KimattaError_Storage(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KimattaError_Planning extends KimattaError {
  const KimattaError_Planning({required this.message}): super._();
  

 final  String message;

/// Create a copy of KimattaError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KimattaError_PlanningCopyWith<KimattaError_Planning> get copyWith => _$KimattaError_PlanningCopyWithImpl<KimattaError_Planning>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KimattaError_Planning&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KimattaError.planning(message: $message)';
}


}

/// @nodoc
abstract mixin class $KimattaError_PlanningCopyWith<$Res> implements $KimattaErrorCopyWith<$Res> {
  factory $KimattaError_PlanningCopyWith(KimattaError_Planning value, $Res Function(KimattaError_Planning) _then) = _$KimattaError_PlanningCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KimattaError_PlanningCopyWithImpl<$Res>
    implements $KimattaError_PlanningCopyWith<$Res> {
  _$KimattaError_PlanningCopyWithImpl(this._self, this._then);

  final KimattaError_Planning _self;
  final $Res Function(KimattaError_Planning) _then;

/// Create a copy of KimattaError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KimattaError_Planning(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KimattaError_Recipe extends KimattaError {
  const KimattaError_Recipe({required this.message}): super._();
  

 final  String message;

/// Create a copy of KimattaError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KimattaError_RecipeCopyWith<KimattaError_Recipe> get copyWith => _$KimattaError_RecipeCopyWithImpl<KimattaError_Recipe>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KimattaError_Recipe&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KimattaError.recipe(message: $message)';
}


}

/// @nodoc
abstract mixin class $KimattaError_RecipeCopyWith<$Res> implements $KimattaErrorCopyWith<$Res> {
  factory $KimattaError_RecipeCopyWith(KimattaError_Recipe value, $Res Function(KimattaError_Recipe) _then) = _$KimattaError_RecipeCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KimattaError_RecipeCopyWithImpl<$Res>
    implements $KimattaError_RecipeCopyWith<$Res> {
  _$KimattaError_RecipeCopyWithImpl(this._self, this._then);

  final KimattaError_Recipe _self;
  final $Res Function(KimattaError_Recipe) _then;

/// Create a copy of KimattaError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KimattaError_Recipe(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KimattaError_Restriction extends KimattaError {
  const KimattaError_Restriction({required this.message}): super._();
  

 final  String message;

/// Create a copy of KimattaError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KimattaError_RestrictionCopyWith<KimattaError_Restriction> get copyWith => _$KimattaError_RestrictionCopyWithImpl<KimattaError_Restriction>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KimattaError_Restriction&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KimattaError.restriction(message: $message)';
}


}

/// @nodoc
abstract mixin class $KimattaError_RestrictionCopyWith<$Res> implements $KimattaErrorCopyWith<$Res> {
  factory $KimattaError_RestrictionCopyWith(KimattaError_Restriction value, $Res Function(KimattaError_Restriction) _then) = _$KimattaError_RestrictionCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KimattaError_RestrictionCopyWithImpl<$Res>
    implements $KimattaError_RestrictionCopyWith<$Res> {
  _$KimattaError_RestrictionCopyWithImpl(this._self, this._then);

  final KimattaError_Restriction _self;
  final $Res Function(KimattaError_Restriction) _then;

/// Create a copy of KimattaError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KimattaError_Restriction(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
