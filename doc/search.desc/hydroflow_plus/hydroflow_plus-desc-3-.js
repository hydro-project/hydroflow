searchState.loadedDescShard("hydroflow_plus", 3, "Helper trait for splitting a variadic into two parts. …\nThe second part when splitting this variadic by <code>Prefix</code>.\nThe un-referenced variadic. Each item will have one layer …\nIndividual variadic items without the Vec wrapper\nA variadic tuple list.\nExtension methods/types for <code>Variadic</code>s.\ntrait for Variadic of vecs, as formed by …\nConvert an exclusive (<code>mut</code>) reference to this variadic into …\nConvert a reference to this variadic into a variadic of …\nClone a variadic of references <code>AsRefVar</code> into a variadic of …\nCopy self per-value.\nTurns into a Drain of items <code>UnVec</code> – i.e. iterate through …\n<code>PartialEq</code> between a referenced variadic and a variadic of …\n<code>PartialEq</code> for the <code>AsRefVar</code> version op <code>Self</code>.\nExtends this variadic value by appending <code>suffix</code> onto the …\nReturns a reference to an element.\nget the unvec’ed Variadic at position <code>index</code>\nReturns an exclusive reference to an element.\nTurns this <code>HomogenousVariadic&lt;T&gt;</code> into an iterator of items …\nwrap all elements of the variadic in `Option``\nwrap all elements of the variadic in a <code>Vec</code>\nTurns into an iterator of items <code>UnVec</code> – i.e. iterate …\nChecks if this variadic type is empty.\nIterate this variadic as <code>&amp;mut dyn Any</code> exclusive references.\nIterate this variadic as <code>&amp;dyn Any</code> references.\nThe length of this variadic type\nConvert all exclusive (<code>mut</code>) references into shared …\nappend an unvec’ed Variadic into this VariadicVec\nReverses this variadic value.\nReverses an AsRefVar variadic value\nSplits this variadic into two parts, first the <code>Prefix</code>, and …\nSplits this variadic into two parts, first the <code>Prefix</code>, and …\nSplits a refvar variadic\nSplits a refvar variadic\nconvert entries to <code>&lt;UnRefVar as VariadicExt&gt;::AsRefVar</code>\nVariadic patterns macro.\nVariadic expressions (values) macro.\nVariadic types macro.\nmodule of collection types for variadics\nThis macro generates a basic variadic trait where each …\nzip across all the vecs in this VariadicVec\nIterator helper for <code>VariadicCountedHashSet::into_iter</code>.\nThe Schema (aka Variadic type) associated with tuples in …\nTrait for a set of Variadic Tuples\nColumn storage for Variadic tuples of type Schema An …\nHashMap keyed on Variadics of (owned value, count) pairs, …\nHashSet that stores Variadics of owned values but allows …\nTrait for a multiset of Tuples\ntrait for sets or multisets of variadics\nCheck for containment\niterate and drain items from the set without deallocating …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\ngiven a RefVariadic lookup key, get a RefVariadic version …\ngiven a RefVariadic lookup key, get a RefVariadic version …\nInsert an element into the set, return true if successful …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturn true if empty\nIterate over the elements of the set\nReturn number of elements in the set\nCreates a new <code>VariadicHashSet</code> with a default hasher.\nCreates a new <code>VariadicCountedHashSet</code> with a default hasher.\ninitialize an empty columnar multiset\nallocate a new VariadicHashSet with a specific hasher and …\nallocate a new VariadicCountedHashSet with a specific …\nallocate a new VariadicHashSet with a specific hasher\nallocate a new VariadicCountedHashSet with a specific …\nA <code>Duration</code> type to represent a span of time, typically …\nA measurement of a monotonically nondecreasing clock. …\nThe maximum duration.\nThe duration of one microsecond.\nThe duration of one millisecond.\nThe duration of one nanosecond.\nThe duration of one second.\nA measurement of the system clock, useful for talking to …\nAn error returned from the <code>duration_since</code> and <code>elapsed</code> …\nAn error which can be returned when converting a …\nAn anchor in time which can be used to create new …\nAn anchor in time which can be used to create new …\nA duration of zero time.\nComputes the absolute difference between <code>self</code> and <code>other</code>.\nPanics\nPanics\nReturns the total number of whole microseconds contained …\nReturns the total number of whole milliseconds contained …\nReturns the number of milliseconds contained by this …\nReturns the number of milliseconds contained by this …\nReturns the total number of nanoseconds contained by this …\nReturns the number of <em>whole</em> seconds contained by this …\nReturns the number of seconds contained by this <code>Duration</code> …\nReturns the number of seconds contained by this <code>Duration</code> …\nReturns <code>Some(t)</code> where <code>t</code> is the time <code>self + duration</code> if <code>t</code> …\nReturns <code>Some(t)</code> where <code>t</code> is the time <code>self + duration</code> if <code>t</code> …\nChecked <code>Duration</code> addition. Computes <code>self + other</code>, …\nChecked <code>Duration</code> division. Computes <code>self / other</code>, …\nReturns the amount of time elapsed from another instant to …\nChecked <code>Duration</code> multiplication. Computes <code>self * other</code>, …\nReturns <code>Some(t)</code> where <code>t</code> is the time <code>self - duration</code> if <code>t</code> …\nReturns <code>Some(t)</code> where <code>t</code> is the time <code>self - duration</code> if <code>t</code> …\nChecked <code>Duration</code> subtraction. Computes <code>self - other</code>, …\nDivides <code>Duration</code> by <code>Duration</code> and returns <code>f32</code>.\nDivides <code>Duration</code> by <code>Duration</code> and returns <code>f64</code>.\nDivides <code>Duration</code> by <code>f32</code>.\nDivides <code>Duration</code> by <code>f64</code>.\nReturns the positive duration which represents how far …\nReturns the amount of time elapsed from another instant to …\nReturns the amount of time elapsed from an earlier point …\nReturns the amount of time elapsed since this instant.\nReturns the difference from this system time to the …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCreates a new <code>Duration</code> from the specified number of days.\nCreates a new <code>Duration</code> from the specified number of hours.\nCreates a new <code>Duration</code> from the specified number of …\nCreates a new <code>Duration</code> from the specified number of …\nCreates a new <code>Duration</code> from the specified number of …\nCreates a new <code>Duration</code> from the specified number of …\nCreates a new <code>Duration</code> from the specified number of whole …\nCreates a new <code>Duration</code> from the specified number of …\nCreates a new <code>Duration</code> from the specified number of …\nCreates a new <code>Duration</code> from the specified number of weeks.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns true if this <code>Duration</code> spans no time.\nMultiplies <code>Duration</code> by <code>f32</code>.\nMultiplies <code>Duration</code> by <code>f64</code>.\nCreates a new <code>Duration</code> from the specified number of whole …\nReturns an instant corresponding to “now”.\nReturns the system time corresponding to “now”.\nSaturating <code>Duration</code> addition. Computes <code>self + other</code>, …\nReturns the amount of time elapsed from another instant to …\nSaturating <code>Duration</code> multiplication. Computes <code>self * other</code>, …\nSaturating <code>Duration</code> subtraction. Computes <code>self - other</code>, …\nReturns the amount of time elapsed from another instant to …\nReturns the fractional part of this <code>Duration</code>, in whole …\nReturns the fractional part of this <code>Duration</code>, in whole …\nReturns the fractional part of this <code>Duration</code>, in …\nThe checked version of <code>from_secs_f32</code>.\nThe checked version of <code>from_secs_f64</code>.")