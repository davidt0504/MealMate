// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'planning_drafts.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DraftActionDto {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftActionDto);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DraftActionDto()';
}


}

/// @nodoc
class $DraftActionDtoCopyWith<$Res>  {
$DraftActionDtoCopyWith(DraftActionDto _, $Res Function(DraftActionDto) __);
}


/// Adds pattern-matching-related methods to [DraftActionDto].
extension DraftActionDtoPatterns on DraftActionDto {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DraftActionDto_Another value)?  another,TResult Function( DraftActionDto_Alternatives value)?  alternatives,TResult Function( DraftActionDto_SetCommitment value)?  setCommitment,TResult Function( DraftActionDto_Choose value)?  choose,TResult Function( DraftActionDto_Undo value)?  undo,TResult Function( DraftActionDto_Discard value)?  discard,TResult Function( DraftActionDto_Reconsider value)?  reconsider,TResult Function( DraftActionDto_Review value)?  review,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DraftActionDto_Another() when another != null:
return another(_that);case DraftActionDto_Alternatives() when alternatives != null:
return alternatives(_that);case DraftActionDto_SetCommitment() when setCommitment != null:
return setCommitment(_that);case DraftActionDto_Choose() when choose != null:
return choose(_that);case DraftActionDto_Undo() when undo != null:
return undo(_that);case DraftActionDto_Discard() when discard != null:
return discard(_that);case DraftActionDto_Reconsider() when reconsider != null:
return reconsider(_that);case DraftActionDto_Review() when review != null:
return review(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DraftActionDto_Another value)  another,required TResult Function( DraftActionDto_Alternatives value)  alternatives,required TResult Function( DraftActionDto_SetCommitment value)  setCommitment,required TResult Function( DraftActionDto_Choose value)  choose,required TResult Function( DraftActionDto_Undo value)  undo,required TResult Function( DraftActionDto_Discard value)  discard,required TResult Function( DraftActionDto_Reconsider value)  reconsider,required TResult Function( DraftActionDto_Review value)  review,}){
final _that = this;
switch (_that) {
case DraftActionDto_Another():
return another(_that);case DraftActionDto_Alternatives():
return alternatives(_that);case DraftActionDto_SetCommitment():
return setCommitment(_that);case DraftActionDto_Choose():
return choose(_that);case DraftActionDto_Undo():
return undo(_that);case DraftActionDto_Discard():
return discard(_that);case DraftActionDto_Reconsider():
return reconsider(_that);case DraftActionDto_Review():
return review(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DraftActionDto_Another value)?  another,TResult? Function( DraftActionDto_Alternatives value)?  alternatives,TResult? Function( DraftActionDto_SetCommitment value)?  setCommitment,TResult? Function( DraftActionDto_Choose value)?  choose,TResult? Function( DraftActionDto_Undo value)?  undo,TResult? Function( DraftActionDto_Discard value)?  discard,TResult? Function( DraftActionDto_Reconsider value)?  reconsider,TResult? Function( DraftActionDto_Review value)?  review,}){
final _that = this;
switch (_that) {
case DraftActionDto_Another() when another != null:
return another(_that);case DraftActionDto_Alternatives() when alternatives != null:
return alternatives(_that);case DraftActionDto_SetCommitment() when setCommitment != null:
return setCommitment(_that);case DraftActionDto_Choose() when choose != null:
return choose(_that);case DraftActionDto_Undo() when undo != null:
return undo(_that);case DraftActionDto_Discard() when discard != null:
return discard(_that);case DraftActionDto_Reconsider() when reconsider != null:
return reconsider(_that);case DraftActionDto_Review() when review != null:
return review(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String date,  MealSlotDto slot)?  another,TResult Function()?  alternatives,TResult Function( String date,  MealSlotDto slot,  bool locked)?  setCommitment,TResult Function( String date,  MealSlotDto slot,  List<MealComponentDto> components,  bool explicitReplace)?  choose,TResult Function()?  undo,TResult Function()?  discard,TResult Function( String? date,  MealSlotDto? slot)?  reconsider,TResult Function( List<ReviewResolutionDto> resolutions)?  review,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DraftActionDto_Another() when another != null:
return another(_that.date,_that.slot);case DraftActionDto_Alternatives() when alternatives != null:
return alternatives();case DraftActionDto_SetCommitment() when setCommitment != null:
return setCommitment(_that.date,_that.slot,_that.locked);case DraftActionDto_Choose() when choose != null:
return choose(_that.date,_that.slot,_that.components,_that.explicitReplace);case DraftActionDto_Undo() when undo != null:
return undo();case DraftActionDto_Discard() when discard != null:
return discard();case DraftActionDto_Reconsider() when reconsider != null:
return reconsider(_that.date,_that.slot);case DraftActionDto_Review() when review != null:
return review(_that.resolutions);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String date,  MealSlotDto slot)  another,required TResult Function()  alternatives,required TResult Function( String date,  MealSlotDto slot,  bool locked)  setCommitment,required TResult Function( String date,  MealSlotDto slot,  List<MealComponentDto> components,  bool explicitReplace)  choose,required TResult Function()  undo,required TResult Function()  discard,required TResult Function( String? date,  MealSlotDto? slot)  reconsider,required TResult Function( List<ReviewResolutionDto> resolutions)  review,}) {final _that = this;
switch (_that) {
case DraftActionDto_Another():
return another(_that.date,_that.slot);case DraftActionDto_Alternatives():
return alternatives();case DraftActionDto_SetCommitment():
return setCommitment(_that.date,_that.slot,_that.locked);case DraftActionDto_Choose():
return choose(_that.date,_that.slot,_that.components,_that.explicitReplace);case DraftActionDto_Undo():
return undo();case DraftActionDto_Discard():
return discard();case DraftActionDto_Reconsider():
return reconsider(_that.date,_that.slot);case DraftActionDto_Review():
return review(_that.resolutions);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String date,  MealSlotDto slot)?  another,TResult? Function()?  alternatives,TResult? Function( String date,  MealSlotDto slot,  bool locked)?  setCommitment,TResult? Function( String date,  MealSlotDto slot,  List<MealComponentDto> components,  bool explicitReplace)?  choose,TResult? Function()?  undo,TResult? Function()?  discard,TResult? Function( String? date,  MealSlotDto? slot)?  reconsider,TResult? Function( List<ReviewResolutionDto> resolutions)?  review,}) {final _that = this;
switch (_that) {
case DraftActionDto_Another() when another != null:
return another(_that.date,_that.slot);case DraftActionDto_Alternatives() when alternatives != null:
return alternatives();case DraftActionDto_SetCommitment() when setCommitment != null:
return setCommitment(_that.date,_that.slot,_that.locked);case DraftActionDto_Choose() when choose != null:
return choose(_that.date,_that.slot,_that.components,_that.explicitReplace);case DraftActionDto_Undo() when undo != null:
return undo();case DraftActionDto_Discard() when discard != null:
return discard();case DraftActionDto_Reconsider() when reconsider != null:
return reconsider(_that.date,_that.slot);case DraftActionDto_Review() when review != null:
return review(_that.resolutions);case _:
  return null;

}
}

}

/// @nodoc


class DraftActionDto_Another extends DraftActionDto {
  const DraftActionDto_Another({required this.date, required this.slot}): super._();
  

 final  String date;
 final  MealSlotDto slot;

/// Create a copy of DraftActionDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DraftActionDto_AnotherCopyWith<DraftActionDto_Another> get copyWith => _$DraftActionDto_AnotherCopyWithImpl<DraftActionDto_Another>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftActionDto_Another&&(identical(other.date, date) || other.date == date)&&(identical(other.slot, slot) || other.slot == slot));
}


@override
int get hashCode => Object.hash(runtimeType,date,slot);

@override
String toString() {
  return 'DraftActionDto.another(date: $date, slot: $slot)';
}


}

/// @nodoc
abstract mixin class $DraftActionDto_AnotherCopyWith<$Res> implements $DraftActionDtoCopyWith<$Res> {
  factory $DraftActionDto_AnotherCopyWith(DraftActionDto_Another value, $Res Function(DraftActionDto_Another) _then) = _$DraftActionDto_AnotherCopyWithImpl;
@useResult
$Res call({
 String date, MealSlotDto slot
});




}
/// @nodoc
class _$DraftActionDto_AnotherCopyWithImpl<$Res>
    implements $DraftActionDto_AnotherCopyWith<$Res> {
  _$DraftActionDto_AnotherCopyWithImpl(this._self, this._then);

  final DraftActionDto_Another _self;
  final $Res Function(DraftActionDto_Another) _then;

/// Create a copy of DraftActionDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? date = null,Object? slot = null,}) {
  return _then(DraftActionDto_Another(
date: null == date ? _self.date : date // ignore: cast_nullable_to_non_nullable
as String,slot: null == slot ? _self.slot : slot // ignore: cast_nullable_to_non_nullable
as MealSlotDto,
  ));
}


}

/// @nodoc


class DraftActionDto_Alternatives extends DraftActionDto {
  const DraftActionDto_Alternatives(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftActionDto_Alternatives);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DraftActionDto.alternatives()';
}


}




/// @nodoc


class DraftActionDto_SetCommitment extends DraftActionDto {
  const DraftActionDto_SetCommitment({required this.date, required this.slot, required this.locked}): super._();
  

 final  String date;
 final  MealSlotDto slot;
 final  bool locked;

/// Create a copy of DraftActionDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DraftActionDto_SetCommitmentCopyWith<DraftActionDto_SetCommitment> get copyWith => _$DraftActionDto_SetCommitmentCopyWithImpl<DraftActionDto_SetCommitment>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftActionDto_SetCommitment&&(identical(other.date, date) || other.date == date)&&(identical(other.slot, slot) || other.slot == slot)&&(identical(other.locked, locked) || other.locked == locked));
}


@override
int get hashCode => Object.hash(runtimeType,date,slot,locked);

@override
String toString() {
  return 'DraftActionDto.setCommitment(date: $date, slot: $slot, locked: $locked)';
}


}

/// @nodoc
abstract mixin class $DraftActionDto_SetCommitmentCopyWith<$Res> implements $DraftActionDtoCopyWith<$Res> {
  factory $DraftActionDto_SetCommitmentCopyWith(DraftActionDto_SetCommitment value, $Res Function(DraftActionDto_SetCommitment) _then) = _$DraftActionDto_SetCommitmentCopyWithImpl;
@useResult
$Res call({
 String date, MealSlotDto slot, bool locked
});




}
/// @nodoc
class _$DraftActionDto_SetCommitmentCopyWithImpl<$Res>
    implements $DraftActionDto_SetCommitmentCopyWith<$Res> {
  _$DraftActionDto_SetCommitmentCopyWithImpl(this._self, this._then);

  final DraftActionDto_SetCommitment _self;
  final $Res Function(DraftActionDto_SetCommitment) _then;

/// Create a copy of DraftActionDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? date = null,Object? slot = null,Object? locked = null,}) {
  return _then(DraftActionDto_SetCommitment(
date: null == date ? _self.date : date // ignore: cast_nullable_to_non_nullable
as String,slot: null == slot ? _self.slot : slot // ignore: cast_nullable_to_non_nullable
as MealSlotDto,locked: null == locked ? _self.locked : locked // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class DraftActionDto_Choose extends DraftActionDto {
  const DraftActionDto_Choose({required this.date, required this.slot, required  List<MealComponentDto> components, required this.explicitReplace}): _components = components,super._();
  

 final  String date;
 final  MealSlotDto slot;
 final  List<MealComponentDto> _components;
 List<MealComponentDto> get components {
  if (_components is EqualUnmodifiableListView) return _components;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_components);
}

 final  bool explicitReplace;

/// Create a copy of DraftActionDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DraftActionDto_ChooseCopyWith<DraftActionDto_Choose> get copyWith => _$DraftActionDto_ChooseCopyWithImpl<DraftActionDto_Choose>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftActionDto_Choose&&(identical(other.date, date) || other.date == date)&&(identical(other.slot, slot) || other.slot == slot)&&const DeepCollectionEquality().equals(other._components, _components)&&(identical(other.explicitReplace, explicitReplace) || other.explicitReplace == explicitReplace));
}


@override
int get hashCode => Object.hash(runtimeType,date,slot,const DeepCollectionEquality().hash(_components),explicitReplace);

@override
String toString() {
  return 'DraftActionDto.choose(date: $date, slot: $slot, components: $components, explicitReplace: $explicitReplace)';
}


}

/// @nodoc
abstract mixin class $DraftActionDto_ChooseCopyWith<$Res> implements $DraftActionDtoCopyWith<$Res> {
  factory $DraftActionDto_ChooseCopyWith(DraftActionDto_Choose value, $Res Function(DraftActionDto_Choose) _then) = _$DraftActionDto_ChooseCopyWithImpl;
@useResult
$Res call({
 String date, MealSlotDto slot, List<MealComponentDto> components, bool explicitReplace
});




}
/// @nodoc
class _$DraftActionDto_ChooseCopyWithImpl<$Res>
    implements $DraftActionDto_ChooseCopyWith<$Res> {
  _$DraftActionDto_ChooseCopyWithImpl(this._self, this._then);

  final DraftActionDto_Choose _self;
  final $Res Function(DraftActionDto_Choose) _then;

/// Create a copy of DraftActionDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? date = null,Object? slot = null,Object? components = null,Object? explicitReplace = null,}) {
  return _then(DraftActionDto_Choose(
date: null == date ? _self.date : date // ignore: cast_nullable_to_non_nullable
as String,slot: null == slot ? _self.slot : slot // ignore: cast_nullable_to_non_nullable
as MealSlotDto,components: null == components ? _self._components : components // ignore: cast_nullable_to_non_nullable
as List<MealComponentDto>,explicitReplace: null == explicitReplace ? _self.explicitReplace : explicitReplace // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class DraftActionDto_Undo extends DraftActionDto {
  const DraftActionDto_Undo(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftActionDto_Undo);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DraftActionDto.undo()';
}


}




/// @nodoc


class DraftActionDto_Discard extends DraftActionDto {
  const DraftActionDto_Discard(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftActionDto_Discard);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DraftActionDto.discard()';
}


}




/// @nodoc


class DraftActionDto_Reconsider extends DraftActionDto {
  const DraftActionDto_Reconsider({this.date, this.slot}): super._();
  

 final  String? date;
 final  MealSlotDto? slot;

/// Create a copy of DraftActionDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DraftActionDto_ReconsiderCopyWith<DraftActionDto_Reconsider> get copyWith => _$DraftActionDto_ReconsiderCopyWithImpl<DraftActionDto_Reconsider>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftActionDto_Reconsider&&(identical(other.date, date) || other.date == date)&&(identical(other.slot, slot) || other.slot == slot));
}


@override
int get hashCode => Object.hash(runtimeType,date,slot);

@override
String toString() {
  return 'DraftActionDto.reconsider(date: $date, slot: $slot)';
}


}

/// @nodoc
abstract mixin class $DraftActionDto_ReconsiderCopyWith<$Res> implements $DraftActionDtoCopyWith<$Res> {
  factory $DraftActionDto_ReconsiderCopyWith(DraftActionDto_Reconsider value, $Res Function(DraftActionDto_Reconsider) _then) = _$DraftActionDto_ReconsiderCopyWithImpl;
@useResult
$Res call({
 String? date, MealSlotDto? slot
});




}
/// @nodoc
class _$DraftActionDto_ReconsiderCopyWithImpl<$Res>
    implements $DraftActionDto_ReconsiderCopyWith<$Res> {
  _$DraftActionDto_ReconsiderCopyWithImpl(this._self, this._then);

  final DraftActionDto_Reconsider _self;
  final $Res Function(DraftActionDto_Reconsider) _then;

/// Create a copy of DraftActionDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? date = freezed,Object? slot = freezed,}) {
  return _then(DraftActionDto_Reconsider(
date: freezed == date ? _self.date : date // ignore: cast_nullable_to_non_nullable
as String?,slot: freezed == slot ? _self.slot : slot // ignore: cast_nullable_to_non_nullable
as MealSlotDto?,
  ));
}


}

/// @nodoc


class DraftActionDto_Review extends DraftActionDto {
  const DraftActionDto_Review({required  List<ReviewResolutionDto> resolutions}): _resolutions = resolutions,super._();
  

 final  List<ReviewResolutionDto> _resolutions;
 List<ReviewResolutionDto> get resolutions {
  if (_resolutions is EqualUnmodifiableListView) return _resolutions;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_resolutions);
}


/// Create a copy of DraftActionDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DraftActionDto_ReviewCopyWith<DraftActionDto_Review> get copyWith => _$DraftActionDto_ReviewCopyWithImpl<DraftActionDto_Review>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftActionDto_Review&&const DeepCollectionEquality().equals(other._resolutions, _resolutions));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_resolutions));

@override
String toString() {
  return 'DraftActionDto.review(resolutions: $resolutions)';
}


}

/// @nodoc
abstract mixin class $DraftActionDto_ReviewCopyWith<$Res> implements $DraftActionDtoCopyWith<$Res> {
  factory $DraftActionDto_ReviewCopyWith(DraftActionDto_Review value, $Res Function(DraftActionDto_Review) _then) = _$DraftActionDto_ReviewCopyWithImpl;
@useResult
$Res call({
 List<ReviewResolutionDto> resolutions
});




}
/// @nodoc
class _$DraftActionDto_ReviewCopyWithImpl<$Res>
    implements $DraftActionDto_ReviewCopyWith<$Res> {
  _$DraftActionDto_ReviewCopyWithImpl(this._self, this._then);

  final DraftActionDto_Review _self;
  final $Res Function(DraftActionDto_Review) _then;

/// Create a copy of DraftActionDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? resolutions = null,}) {
  return _then(DraftActionDto_Review(
resolutions: null == resolutions ? _self._resolutions : resolutions // ignore: cast_nullable_to_non_nullable
as List<ReviewResolutionDto>,
  ));
}


}

/// @nodoc
mixin _$MealExclusionKindDto {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MealExclusionKindDto);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'MealExclusionKindDto()';
}


}

/// @nodoc
class $MealExclusionKindDtoCopyWith<$Res>  {
$MealExclusionKindDtoCopyWith(MealExclusionKindDto _, $Res Function(MealExclusionKindDto) __);
}


/// Adds pattern-matching-related methods to [MealExclusionKindDto].
extension MealExclusionKindDtoPatterns on MealExclusionKindDto {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( MealExclusionKindDto_Dish value)?  dish,TResult Function( MealExclusionKindDto_Phrase value)?  phrase,required TResult orElse(),}){
final _that = this;
switch (_that) {
case MealExclusionKindDto_Dish() when dish != null:
return dish(_that);case MealExclusionKindDto_Phrase() when phrase != null:
return phrase(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( MealExclusionKindDto_Dish value)  dish,required TResult Function( MealExclusionKindDto_Phrase value)  phrase,}){
final _that = this;
switch (_that) {
case MealExclusionKindDto_Dish():
return dish(_that);case MealExclusionKindDto_Phrase():
return phrase(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( MealExclusionKindDto_Dish value)?  dish,TResult? Function( MealExclusionKindDto_Phrase value)?  phrase,}){
final _that = this;
switch (_that) {
case MealExclusionKindDto_Dish() when dish != null:
return dish(_that);case MealExclusionKindDto_Phrase() when phrase != null:
return phrase(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String identity,  String? title,  bool available)?  dish,TResult Function( String subject)?  phrase,required TResult orElse(),}) {final _that = this;
switch (_that) {
case MealExclusionKindDto_Dish() when dish != null:
return dish(_that.identity,_that.title,_that.available);case MealExclusionKindDto_Phrase() when phrase != null:
return phrase(_that.subject);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String identity,  String? title,  bool available)  dish,required TResult Function( String subject)  phrase,}) {final _that = this;
switch (_that) {
case MealExclusionKindDto_Dish():
return dish(_that.identity,_that.title,_that.available);case MealExclusionKindDto_Phrase():
return phrase(_that.subject);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String identity,  String? title,  bool available)?  dish,TResult? Function( String subject)?  phrase,}) {final _that = this;
switch (_that) {
case MealExclusionKindDto_Dish() when dish != null:
return dish(_that.identity,_that.title,_that.available);case MealExclusionKindDto_Phrase() when phrase != null:
return phrase(_that.subject);case _:
  return null;

}
}

}

/// @nodoc


class MealExclusionKindDto_Dish extends MealExclusionKindDto {
  const MealExclusionKindDto_Dish({required this.identity, this.title, required this.available}): super._();
  

 final  String identity;
 final  String? title;
 final  bool available;

/// Create a copy of MealExclusionKindDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MealExclusionKindDto_DishCopyWith<MealExclusionKindDto_Dish> get copyWith => _$MealExclusionKindDto_DishCopyWithImpl<MealExclusionKindDto_Dish>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MealExclusionKindDto_Dish&&(identical(other.identity, identity) || other.identity == identity)&&(identical(other.title, title) || other.title == title)&&(identical(other.available, available) || other.available == available));
}


@override
int get hashCode => Object.hash(runtimeType,identity,title,available);

@override
String toString() {
  return 'MealExclusionKindDto.dish(identity: $identity, title: $title, available: $available)';
}


}

/// @nodoc
abstract mixin class $MealExclusionKindDto_DishCopyWith<$Res> implements $MealExclusionKindDtoCopyWith<$Res> {
  factory $MealExclusionKindDto_DishCopyWith(MealExclusionKindDto_Dish value, $Res Function(MealExclusionKindDto_Dish) _then) = _$MealExclusionKindDto_DishCopyWithImpl;
@useResult
$Res call({
 String identity, String? title, bool available
});




}
/// @nodoc
class _$MealExclusionKindDto_DishCopyWithImpl<$Res>
    implements $MealExclusionKindDto_DishCopyWith<$Res> {
  _$MealExclusionKindDto_DishCopyWithImpl(this._self, this._then);

  final MealExclusionKindDto_Dish _self;
  final $Res Function(MealExclusionKindDto_Dish) _then;

/// Create a copy of MealExclusionKindDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? identity = null,Object? title = freezed,Object? available = null,}) {
  return _then(MealExclusionKindDto_Dish(
identity: null == identity ? _self.identity : identity // ignore: cast_nullable_to_non_nullable
as String,title: freezed == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String?,available: null == available ? _self.available : available // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class MealExclusionKindDto_Phrase extends MealExclusionKindDto {
  const MealExclusionKindDto_Phrase({required this.subject}): super._();
  

 final  String subject;

/// Create a copy of MealExclusionKindDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MealExclusionKindDto_PhraseCopyWith<MealExclusionKindDto_Phrase> get copyWith => _$MealExclusionKindDto_PhraseCopyWithImpl<MealExclusionKindDto_Phrase>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MealExclusionKindDto_Phrase&&(identical(other.subject, subject) || other.subject == subject));
}


@override
int get hashCode => Object.hash(runtimeType,subject);

@override
String toString() {
  return 'MealExclusionKindDto.phrase(subject: $subject)';
}


}

/// @nodoc
abstract mixin class $MealExclusionKindDto_PhraseCopyWith<$Res> implements $MealExclusionKindDtoCopyWith<$Res> {
  factory $MealExclusionKindDto_PhraseCopyWith(MealExclusionKindDto_Phrase value, $Res Function(MealExclusionKindDto_Phrase) _then) = _$MealExclusionKindDto_PhraseCopyWithImpl;
@useResult
$Res call({
 String subject
});




}
/// @nodoc
class _$MealExclusionKindDto_PhraseCopyWithImpl<$Res>
    implements $MealExclusionKindDto_PhraseCopyWith<$Res> {
  _$MealExclusionKindDto_PhraseCopyWithImpl(this._self, this._then);

  final MealExclusionKindDto_Phrase _self;
  final $Res Function(MealExclusionKindDto_Phrase) _then;

/// Create a copy of MealExclusionKindDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? subject = null,}) {
  return _then(MealExclusionKindDto_Phrase(
subject: null == subject ? _self.subject : subject // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
