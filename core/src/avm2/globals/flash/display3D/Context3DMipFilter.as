// The initial version of this file was autogenerated from the official AS3 reference at
// https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display3D/Context3DMipFilter.html
// by https://github.com/golfinq/ActionScript_Event_Builder
// It won't be regenerated in the future, so feel free to edit and/or fix

package flash.display3D
{

    public final class Context3DMipFilter
    {
        // Select the two closest MIP levels and linearly blend between them (the highest quality mode, but has some performance cost).
        public static const MIPLINEAR:String = "miplinear";

        // Use the nearest neighbor metric to select MIP levels (the fastest rendering method).
        public static const MIPNEAREST:String = "mipnearest";

        // Always use the top level texture (has a performance penalty when downscaling).
        public static const MIPNONE:String = "mipnone";

    }
}
