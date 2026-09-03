// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'decisions.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PlanDecisionDto {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlanDecisionDto);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlanDecisionDto()';
}


}

/// @nodoc
class $PlanDecisionDtoCopyWith<$Res>  {
$PlanDecisionDtoCopyWith(PlanDecisionDto _, $Res Function(PlanDecisionDto) __);
}


/// Adds pattern-matching-related methods to [PlanDecisionDto].
extension PlanDecisionDtoPatterns on PlanDecisionDto {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PlanDecisionDto_Swap value)?  swap,TResult Function( PlanDecisionDto_Veto value)?  veto,TResult Function( PlanDecisionDto_RestrictionsReviewed value)?  restrictionsReviewed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PlanDecisionDto_Swap() when swap != null:
return swap(_that);case PlanDecisionDto_Veto() when veto != null:
return veto(_that);case PlanDecisionDto_RestrictionsReviewed() when restrictionsReviewed != null:
return restrictionsReviewed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PlanDecisionDto_Swap value)  swap,required TResult Function( PlanDecisionDto_Veto value)  veto,required TResult Function( PlanDecisionDto_RestrictionsReviewed value)  restrictionsReviewed,}){
final _that = this;
switch (_that) {
case PlanDecisionDto_Swap():
return swap(_that);case PlanDecisionDto_Veto():
return veto(_that);case PlanDecisionDto_RestrictionsReviewed():
return restrictionsReviewed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PlanDecisionDto_Swap value)?  swap,TResult? Function( PlanDecisionDto_Veto value)?  veto,TResult? Function( PlanDecisionDto_RestrictionsReviewed value)?  restrictionsReviewed,}){
final _that = this;
switch (_that) {
case PlanDecisionDto_Swap() when swap != null:
return swap(_that);case PlanDecisionDto_Veto() when veto != null:
return veto(_that);case PlanDecisionDto_RestrictionsReviewed() when restrictionsReviewed != null:
return restrictionsReviewed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String date,  MealSlotDto slot,  List<MealComponentDto> components)?  swap,TResult Function( String subject,  String date,  MealSlotDto slot)?  veto,TResult Function()?  restrictionsReviewed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PlanDecisionDto_Swap() when swap != null:
return swap(_that.date,_that.slot,_that.components);case PlanDecisionDto_Veto() when veto != null:
return veto(_that.subject,_that.date,_that.slot);case PlanDecisionDto_RestrictionsReviewed() when restrictionsReviewed != null:
return restrictionsReviewed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String date,  MealSlotDto slot,  List<MealComponentDto> components)  swap,required TResult Function( String subject,  String date,  MealSlotDto slot)  veto,required TResult Function()  restrictionsReviewed,}) {final _that = this;
switch (_that) {
case PlanDecisionDto_Swap():
return swap(_that.date,_that.slot,_that.components);case PlanDecisionDto_Veto():
return veto(_that.subject,_that.date,_that.slot);case PlanDecisionDto_RestrictionsReviewed():
return restrictionsReviewed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String date,  MealSlotDto slot,  List<MealComponentDto> components)?  swap,TResult? Function( String subject,  String date,  MealSlotDto slot)?  veto,TResult? Function()?  restrictionsReviewed,}) {final _that = this;
switch (_that) {
case PlanDecisionDto_Swap() when swap != null:
return swap(_that.date,_that.slot,_that.components);case PlanDecisionDto_Veto() when veto != null:
return veto(_that.subject,_that.date,_that.slot);case PlanDecisionDto_RestrictionsReviewed() when restrictionsReviewed != null:
return restrictionsReviewed();case _:
  return null;

}
}

}

/// @nodoc


class PlanDecisionDto_Swap extends PlanDecisionDto {
  const PlanDecisionDto_Swap({required this.date, required this.slot, required  List<MealComponentDto> components}): _components = components,super._();
  

/// ISO civil date.
 final  String date;
 final  MealSlotDto slot;
 final  List<MealComponentDto> _components;
 List<MealComponentDto> get components {
  if (_components is EqualUnmodifiableListView) return _components;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_components);
}


/// Create a copy of PlanDecisionDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlanDecisionDto_SwapCopyWith<PlanDecisionDto_Swap> get copyWith => _$PlanDecisionDto_SwapCopyWithImpl<PlanDecisionDto_Swap>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlanDecisionDto_Swap&&(identical(other.date, date) || other.date == date)&&(identical(other.slot, slot) || other.slot == slot)&&const DeepCollectionEquality().equals(other._components, _components));
}


@override
int get hashCode => Object.hash(runtimeType,date,slot,const DeepCollectionEquality().hash(_components));

@override
String toString() {
  return 'PlanDecisionDto.swap(date: $date, slot: $slot, components: $components)';
}


}

/// @nodoc
abstract mixin class $PlanDecisionDto_SwapCopyWith<$Res> implements $PlanDecisionDtoCopyWith<$Res> {
  factory $PlanDecisionDto_SwapCopyWith(PlanDecisionDto_Swap value, $Res Function(PlanDecisionDto_Swap) _then) = _$PlanDecisionDto_SwapCopyWithImpl;
@useResult
$Res call({
 String date, MealSlotDto slot, List<MealComponentDto> components
});




}
/// @nodoc
class _$PlanDecisionDto_SwapCopyWithImpl<$Res>
    implements $PlanDecisionDto_SwapCopyWith<$Res> {
  _$PlanDecisionDto_SwapCopyWithImpl(this._self, this._then);

  final PlanDecisionDto_Swap _self;
  final $Res Function(PlanDecisionDto_Swap) _then;

/// Create a copy of PlanDecisionDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? date = null,Object? slot = null,Object? components = null,}) {
  return _then(PlanDecisionDto_Swap(
date: null == date ? _self.date : date // ignore: cast_nullable_to_non_nullable
as String,slot: null == slot ? _self.slot : slot // ignore: cast_nullable_to_non_nullable
as MealSlotDto,components: null == components ? _self._components : components // ignore: cast_nullable_to_non_nullable
as List<MealComponentDto>,
  ));
}


}

/// @nodoc


class PlanDecisionDto_Veto extends PlanDecisionDto {
  const PlanDecisionDto_Veto({required this.subject, required this.date, required this.slot}): super._();
  

 final  String subject;
/// ISO civil date of the slot the veto was uttered on, for the ledger only.
 final  String date;
 final  MealSlotDto slot;

/// Create a copy of PlanDecisionDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlanDecisionDto_VetoCopyWith<PlanDecisionDto_Veto> get copyWith => _$PlanDecisionDto_VetoCopyWithImpl<PlanDecisionDto_Veto>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlanDecisionDto_Veto&&(identical(other.subject, subject) || other.subject == subject)&&(identical(other.date, date) || other.date == date)&&(identical(other.slot, slot) || other.slot == slot));
}


@override
int get hashCode => Object.hash(runtimeType,subject,date,slot);

@override
String toString() {
  return 'PlanDecisionDto.veto(subject: $subject, date: $date, slot: $slot)';
}


}

/// @nodoc
abstract mixin class $PlanDecisionDto_VetoCopyWith<$Res> implements $PlanDecisionDtoCopyWith<$Res> {
  factory $PlanDecisionDto_VetoCopyWith(PlanDecisionDto_Veto value, $Res Function(PlanDecisionDto_Veto) _then) = _$PlanDecisionDto_VetoCopyWithImpl;
@useResult
$Res call({
 String subject, String date, MealSlotDto slot
});




}
/// @nodoc
class _$PlanDecisionDto_VetoCopyWithImpl<$Res>
    implements $PlanDecisionDto_VetoCopyWith<$Res> {
  _$PlanDecisionDto_VetoCopyWithImpl(this._self, this._then);

  final PlanDecisionDto_Veto _self;
  final $Res Function(PlanDecisionDto_Veto) _then;

/// Create a copy of PlanDecisionDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? subject = null,Object? date = null,Object? slot = null,}) {
  return _then(PlanDecisionDto_Veto(
subject: null == subject ? _self.subject : subject // ignore: cast_nullable_to_non_nullable
as String,date: null == date ? _self.date : date // ignore: cast_nullable_to_non_nullable
as String,slot: null == slot ? _self.slot : slot // ignore: cast_nullable_to_non_nullable
as MealSlotDto,
  ));
}


}

/// @nodoc


class PlanDecisionDto_RestrictionsReviewed extends PlanDecisionDto {
  const PlanDecisionDto_RestrictionsReviewed(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlanDecisionDto_RestrictionsReviewed);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlanDecisionDto.restrictionsReviewed()';
}


}




// dart format on
