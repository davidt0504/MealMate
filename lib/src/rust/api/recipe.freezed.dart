// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'recipe.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$IngredientRefDto {

 String get id;
/// Create a copy of IngredientRefDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$IngredientRefDtoCopyWith<IngredientRefDto> get copyWith => _$IngredientRefDtoCopyWithImpl<IngredientRefDto>(this as IngredientRefDto, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is IngredientRefDto&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'IngredientRefDto(id: $id)';
}


}

/// @nodoc
abstract mixin class $IngredientRefDtoCopyWith<$Res>  {
  factory $IngredientRefDtoCopyWith(IngredientRefDto value, $Res Function(IngredientRefDto) _then) = _$IngredientRefDtoCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$IngredientRefDtoCopyWithImpl<$Res>
    implements $IngredientRefDtoCopyWith<$Res> {
  _$IngredientRefDtoCopyWithImpl(this._self, this._then);

  final IngredientRefDto _self;
  final $Res Function(IngredientRefDto) _then;

/// Create a copy of IngredientRefDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [IngredientRefDto].
extension IngredientRefDtoPatterns on IngredientRefDto {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( IngredientRefDto_Catalog value)?  catalog,TResult Function( IngredientRefDto_Custom value)?  custom,required TResult orElse(),}){
final _that = this;
switch (_that) {
case IngredientRefDto_Catalog() when catalog != null:
return catalog(_that);case IngredientRefDto_Custom() when custom != null:
return custom(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( IngredientRefDto_Catalog value)  catalog,required TResult Function( IngredientRefDto_Custom value)  custom,}){
final _that = this;
switch (_that) {
case IngredientRefDto_Catalog():
return catalog(_that);case IngredientRefDto_Custom():
return custom(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( IngredientRefDto_Catalog value)?  catalog,TResult? Function( IngredientRefDto_Custom value)?  custom,}){
final _that = this;
switch (_that) {
case IngredientRefDto_Catalog() when catalog != null:
return catalog(_that);case IngredientRefDto_Custom() when custom != null:
return custom(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String id)?  catalog,TResult Function( String id)?  custom,required TResult orElse(),}) {final _that = this;
switch (_that) {
case IngredientRefDto_Catalog() when catalog != null:
return catalog(_that.id);case IngredientRefDto_Custom() when custom != null:
return custom(_that.id);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String id)  catalog,required TResult Function( String id)  custom,}) {final _that = this;
switch (_that) {
case IngredientRefDto_Catalog():
return catalog(_that.id);case IngredientRefDto_Custom():
return custom(_that.id);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String id)?  catalog,TResult? Function( String id)?  custom,}) {final _that = this;
switch (_that) {
case IngredientRefDto_Catalog() when catalog != null:
return catalog(_that.id);case IngredientRefDto_Custom() when custom != null:
return custom(_that.id);case _:
  return null;

}
}

}

/// @nodoc


class IngredientRefDto_Catalog extends IngredientRefDto {
  const IngredientRefDto_Catalog({required this.id}): super._();
  

@override final  String id;

/// Create a copy of IngredientRefDto
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$IngredientRefDto_CatalogCopyWith<IngredientRefDto_Catalog> get copyWith => _$IngredientRefDto_CatalogCopyWithImpl<IngredientRefDto_Catalog>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is IngredientRefDto_Catalog&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'IngredientRefDto.catalog(id: $id)';
}


}

/// @nodoc
abstract mixin class $IngredientRefDto_CatalogCopyWith<$Res> implements $IngredientRefDtoCopyWith<$Res> {
  factory $IngredientRefDto_CatalogCopyWith(IngredientRefDto_Catalog value, $Res Function(IngredientRefDto_Catalog) _then) = _$IngredientRefDto_CatalogCopyWithImpl;
@override @useResult
$Res call({
 String id
});




}
/// @nodoc
class _$IngredientRefDto_CatalogCopyWithImpl<$Res>
    implements $IngredientRefDto_CatalogCopyWith<$Res> {
  _$IngredientRefDto_CatalogCopyWithImpl(this._self, this._then);

  final IngredientRefDto_Catalog _self;
  final $Res Function(IngredientRefDto_Catalog) _then;

/// Create a copy of IngredientRefDto
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(IngredientRefDto_Catalog(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class IngredientRefDto_Custom extends IngredientRefDto {
  const IngredientRefDto_Custom({required this.id}): super._();
  

@override final  String id;

/// Create a copy of IngredientRefDto
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$IngredientRefDto_CustomCopyWith<IngredientRefDto_Custom> get copyWith => _$IngredientRefDto_CustomCopyWithImpl<IngredientRefDto_Custom>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is IngredientRefDto_Custom&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'IngredientRefDto.custom(id: $id)';
}


}

/// @nodoc
abstract mixin class $IngredientRefDto_CustomCopyWith<$Res> implements $IngredientRefDtoCopyWith<$Res> {
  factory $IngredientRefDto_CustomCopyWith(IngredientRefDto_Custom value, $Res Function(IngredientRefDto_Custom) _then) = _$IngredientRefDto_CustomCopyWithImpl;
@override @useResult
$Res call({
 String id
});




}
/// @nodoc
class _$IngredientRefDto_CustomCopyWithImpl<$Res>
    implements $IngredientRefDto_CustomCopyWith<$Res> {
  _$IngredientRefDto_CustomCopyWithImpl(this._self, this._then);

  final IngredientRefDto_Custom _self;
  final $Res Function(IngredientRefDto_Custom) _then;

/// Create a copy of IngredientRefDto
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(IngredientRefDto_Custom(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$QuantityDto {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is QuantityDto);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'QuantityDto()';
}


}

/// @nodoc
class $QuantityDtoCopyWith<$Res>  {
$QuantityDtoCopyWith(QuantityDto _, $Res Function(QuantityDto) __);
}


/// Adds pattern-matching-related methods to [QuantityDto].
extension QuantityDtoPatterns on QuantityDto {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( QuantityDto_Unknown value)?  unknown,TResult Function( QuantityDto_Exact value)?  exact,TResult Function( QuantityDto_Range value)?  range,required TResult orElse(),}){
final _that = this;
switch (_that) {
case QuantityDto_Unknown() when unknown != null:
return unknown(_that);case QuantityDto_Exact() when exact != null:
return exact(_that);case QuantityDto_Range() when range != null:
return range(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( QuantityDto_Unknown value)  unknown,required TResult Function( QuantityDto_Exact value)  exact,required TResult Function( QuantityDto_Range value)  range,}){
final _that = this;
switch (_that) {
case QuantityDto_Unknown():
return unknown(_that);case QuantityDto_Exact():
return exact(_that);case QuantityDto_Range():
return range(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( QuantityDto_Unknown value)?  unknown,TResult? Function( QuantityDto_Exact value)?  exact,TResult? Function( QuantityDto_Range value)?  range,}){
final _that = this;
switch (_that) {
case QuantityDto_Unknown() when unknown != null:
return unknown(_that);case QuantityDto_Exact() when exact != null:
return exact(_that);case QuantityDto_Range() when range != null:
return range(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  unknown,TResult Function( int numer,  int denom)?  exact,TResult Function( int minNumer,  int minDenom,  int maxNumer,  int maxDenom)?  range,required TResult orElse(),}) {final _that = this;
switch (_that) {
case QuantityDto_Unknown() when unknown != null:
return unknown();case QuantityDto_Exact() when exact != null:
return exact(_that.numer,_that.denom);case QuantityDto_Range() when range != null:
return range(_that.minNumer,_that.minDenom,_that.maxNumer,_that.maxDenom);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  unknown,required TResult Function( int numer,  int denom)  exact,required TResult Function( int minNumer,  int minDenom,  int maxNumer,  int maxDenom)  range,}) {final _that = this;
switch (_that) {
case QuantityDto_Unknown():
return unknown();case QuantityDto_Exact():
return exact(_that.numer,_that.denom);case QuantityDto_Range():
return range(_that.minNumer,_that.minDenom,_that.maxNumer,_that.maxDenom);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  unknown,TResult? Function( int numer,  int denom)?  exact,TResult? Function( int minNumer,  int minDenom,  int maxNumer,  int maxDenom)?  range,}) {final _that = this;
switch (_that) {
case QuantityDto_Unknown() when unknown != null:
return unknown();case QuantityDto_Exact() when exact != null:
return exact(_that.numer,_that.denom);case QuantityDto_Range() when range != null:
return range(_that.minNumer,_that.minDenom,_that.maxNumer,_that.maxDenom);case _:
  return null;

}
}

}

/// @nodoc


class QuantityDto_Unknown extends QuantityDto {
  const QuantityDto_Unknown(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is QuantityDto_Unknown);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'QuantityDto.unknown()';
}


}




/// @nodoc


class QuantityDto_Exact extends QuantityDto {
  const QuantityDto_Exact({required this.numer, required this.denom}): super._();
  

 final  int numer;
 final  int denom;

/// Create a copy of QuantityDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$QuantityDto_ExactCopyWith<QuantityDto_Exact> get copyWith => _$QuantityDto_ExactCopyWithImpl<QuantityDto_Exact>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is QuantityDto_Exact&&(identical(other.numer, numer) || other.numer == numer)&&(identical(other.denom, denom) || other.denom == denom));
}


@override
int get hashCode => Object.hash(runtimeType,numer,denom);

@override
String toString() {
  return 'QuantityDto.exact(numer: $numer, denom: $denom)';
}


}

/// @nodoc
abstract mixin class $QuantityDto_ExactCopyWith<$Res> implements $QuantityDtoCopyWith<$Res> {
  factory $QuantityDto_ExactCopyWith(QuantityDto_Exact value, $Res Function(QuantityDto_Exact) _then) = _$QuantityDto_ExactCopyWithImpl;
@useResult
$Res call({
 int numer, int denom
});




}
/// @nodoc
class _$QuantityDto_ExactCopyWithImpl<$Res>
    implements $QuantityDto_ExactCopyWith<$Res> {
  _$QuantityDto_ExactCopyWithImpl(this._self, this._then);

  final QuantityDto_Exact _self;
  final $Res Function(QuantityDto_Exact) _then;

/// Create a copy of QuantityDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? numer = null,Object? denom = null,}) {
  return _then(QuantityDto_Exact(
numer: null == numer ? _self.numer : numer // ignore: cast_nullable_to_non_nullable
as int,denom: null == denom ? _self.denom : denom // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class QuantityDto_Range extends QuantityDto {
  const QuantityDto_Range({required this.minNumer, required this.minDenom, required this.maxNumer, required this.maxDenom}): super._();
  

 final  int minNumer;
 final  int minDenom;
 final  int maxNumer;
 final  int maxDenom;

/// Create a copy of QuantityDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$QuantityDto_RangeCopyWith<QuantityDto_Range> get copyWith => _$QuantityDto_RangeCopyWithImpl<QuantityDto_Range>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is QuantityDto_Range&&(identical(other.minNumer, minNumer) || other.minNumer == minNumer)&&(identical(other.minDenom, minDenom) || other.minDenom == minDenom)&&(identical(other.maxNumer, maxNumer) || other.maxNumer == maxNumer)&&(identical(other.maxDenom, maxDenom) || other.maxDenom == maxDenom));
}


@override
int get hashCode => Object.hash(runtimeType,minNumer,minDenom,maxNumer,maxDenom);

@override
String toString() {
  return 'QuantityDto.range(minNumer: $minNumer, minDenom: $minDenom, maxNumer: $maxNumer, maxDenom: $maxDenom)';
}


}

/// @nodoc
abstract mixin class $QuantityDto_RangeCopyWith<$Res> implements $QuantityDtoCopyWith<$Res> {
  factory $QuantityDto_RangeCopyWith(QuantityDto_Range value, $Res Function(QuantityDto_Range) _then) = _$QuantityDto_RangeCopyWithImpl;
@useResult
$Res call({
 int minNumer, int minDenom, int maxNumer, int maxDenom
});




}
/// @nodoc
class _$QuantityDto_RangeCopyWithImpl<$Res>
    implements $QuantityDto_RangeCopyWith<$Res> {
  _$QuantityDto_RangeCopyWithImpl(this._self, this._then);

  final QuantityDto_Range _self;
  final $Res Function(QuantityDto_Range) _then;

/// Create a copy of QuantityDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? minNumer = null,Object? minDenom = null,Object? maxNumer = null,Object? maxDenom = null,}) {
  return _then(QuantityDto_Range(
minNumer: null == minNumer ? _self.minNumer : minNumer // ignore: cast_nullable_to_non_nullable
as int,minDenom: null == minDenom ? _self.minDenom : minDenom // ignore: cast_nullable_to_non_nullable
as int,maxNumer: null == maxNumer ? _self.maxNumer : maxNumer // ignore: cast_nullable_to_non_nullable
as int,maxDenom: null == maxDenom ? _self.maxDenom : maxDenom // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc
mixin _$UnitDto {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnitDto);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'UnitDto()';
}


}

/// @nodoc
class $UnitDtoCopyWith<$Res>  {
$UnitDtoCopyWith(UnitDto _, $Res Function(UnitDto) __);
}


/// Adds pattern-matching-related methods to [UnitDto].
extension UnitDtoPatterns on UnitDto {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( UnitDto_None value)?  none,TResult Function( UnitDto_Known value)?  known,TResult Function( UnitDto_Other value)?  other,required TResult orElse(),}){
final _that = this;
switch (_that) {
case UnitDto_None() when none != null:
return none(_that);case UnitDto_Known() when known != null:
return known(_that);case UnitDto_Other() when other != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( UnitDto_None value)  none,required TResult Function( UnitDto_Known value)  known,required TResult Function( UnitDto_Other value)  other,}){
final _that = this;
switch (_that) {
case UnitDto_None():
return none(_that);case UnitDto_Known():
return known(_that);case UnitDto_Other():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( UnitDto_None value)?  none,TResult? Function( UnitDto_Known value)?  known,TResult? Function( UnitDto_Other value)?  other,}){
final _that = this;
switch (_that) {
case UnitDto_None() when none != null:
return none(_that);case UnitDto_Known() when known != null:
return known(_that);case UnitDto_Other() when other != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  none,TResult Function( String unit)?  known,TResult Function( String text)?  other,required TResult orElse(),}) {final _that = this;
switch (_that) {
case UnitDto_None() when none != null:
return none();case UnitDto_Known() when known != null:
return known(_that.unit);case UnitDto_Other() when other != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  none,required TResult Function( String unit)  known,required TResult Function( String text)  other,}) {final _that = this;
switch (_that) {
case UnitDto_None():
return none();case UnitDto_Known():
return known(_that.unit);case UnitDto_Other():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  none,TResult? Function( String unit)?  known,TResult? Function( String text)?  other,}) {final _that = this;
switch (_that) {
case UnitDto_None() when none != null:
return none();case UnitDto_Known() when known != null:
return known(_that.unit);case UnitDto_Other() when other != null:
return other(_that.text);case _:
  return null;

}
}

}

/// @nodoc


class UnitDto_None extends UnitDto {
  const UnitDto_None(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnitDto_None);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'UnitDto.none()';
}


}




/// @nodoc


class UnitDto_Known extends UnitDto {
  const UnitDto_Known({required this.unit}): super._();
  

 final  String unit;

/// Create a copy of UnitDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UnitDto_KnownCopyWith<UnitDto_Known> get copyWith => _$UnitDto_KnownCopyWithImpl<UnitDto_Known>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnitDto_Known&&(identical(other.unit, unit) || other.unit == unit));
}


@override
int get hashCode => Object.hash(runtimeType,unit);

@override
String toString() {
  return 'UnitDto.known(unit: $unit)';
}


}

/// @nodoc
abstract mixin class $UnitDto_KnownCopyWith<$Res> implements $UnitDtoCopyWith<$Res> {
  factory $UnitDto_KnownCopyWith(UnitDto_Known value, $Res Function(UnitDto_Known) _then) = _$UnitDto_KnownCopyWithImpl;
@useResult
$Res call({
 String unit
});




}
/// @nodoc
class _$UnitDto_KnownCopyWithImpl<$Res>
    implements $UnitDto_KnownCopyWith<$Res> {
  _$UnitDto_KnownCopyWithImpl(this._self, this._then);

  final UnitDto_Known _self;
  final $Res Function(UnitDto_Known) _then;

/// Create a copy of UnitDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? unit = null,}) {
  return _then(UnitDto_Known(
unit: null == unit ? _self.unit : unit // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class UnitDto_Other extends UnitDto {
  const UnitDto_Other({required this.text}): super._();
  

 final  String text;

/// Create a copy of UnitDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UnitDto_OtherCopyWith<UnitDto_Other> get copyWith => _$UnitDto_OtherCopyWithImpl<UnitDto_Other>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnitDto_Other&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'UnitDto.other(text: $text)';
}


}

/// @nodoc
abstract mixin class $UnitDto_OtherCopyWith<$Res> implements $UnitDtoCopyWith<$Res> {
  factory $UnitDto_OtherCopyWith(UnitDto_Other value, $Res Function(UnitDto_Other) _then) = _$UnitDto_OtherCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$UnitDto_OtherCopyWithImpl<$Res>
    implements $UnitDto_OtherCopyWith<$Res> {
  _$UnitDto_OtherCopyWithImpl(this._self, this._then);

  final UnitDto_Other _self;
  final $Res Function(UnitDto_Other) _then;

/// Create a copy of UnitDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(UnitDto_Other(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
